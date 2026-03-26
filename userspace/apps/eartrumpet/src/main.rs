use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation, ListBox, ListBoxRow, Button, Scale, Adjustment, ComboBoxText, ScrolledWindow};
use std::process::Command;
use std::time::Duration;
use serde_json::Value;
use glib::{clone, timeout_add_local};

#[derive(Clone, Debug)]
struct Sink {
    index: i64,
    name: String,
    description: String,
}

#[derive(Clone, Debug)]
struct SinkInput {
    index: i64,
    sink: i64,
    app_name: String,
    media_name: String,
    volume_percent: f64, // 0-150
    mute: bool,
}

fn run_pactl_json(args: &[&str]) -> Option<Value> {
    let out = Command::new("pactl").args(args).output().ok()?;
    if !out.status.success() { return None; }
    serde_json::from_slice(&out.stdout).ok()
}

fn get_default_sink_name() -> Option<String> {
    let out = Command::new("pactl").args(["get-default-sink"]).output().ok()?;
    if !out.status.success() { return None; }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn sinks() -> Vec<Sink> {
    let mut res = Vec::new();
    if let Some(v) = run_pactl_json(&["-f","json","list","sinks"]) {
        if let Some(arr) = v.get("sinks").and_then(|x| x.as_array()) {
            for s in arr {
                let index = s.get("index").and_then(|x| x.as_i64()).unwrap_or(-1);
                let name = s.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                let description = s.get("description").and_then(|x| x.as_str()).unwrap_or("").to_string();
                res.push(Sink{ index, name, description });
            }
        }
    }
    res
}

fn sink_inputs() -> Vec<SinkInput> {
    let mut res = Vec::new();
    if let Some(v) = run_pactl_json(&["-f","json","list","sink-inputs"]) {
        if let Some(arr) = v.get("sink_inputs").and_then(|x| x.as_array()) {
            for i in arr {
                let index = i.get("index").and_then(|x| x.as_i64()).unwrap_or(-1);
                let sink = i.get("sink").and_then(|x| x.as_i64()).unwrap_or(-1);
                let props = i.get("properties").cloned().unwrap_or(Value::Null);
                let app_name = props.get("application.name").and_then(|x| x.as_str()).unwrap_or("?").to_string();
                let media_name = props.get("media.name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                // volume percent from average of channel volumes percent if present
                let mut volume_percent = 0.0;
                if let Some(vol) = i.get("volume") {
                    if let Some(values) = vol.get("values").and_then(|x| x.as_array()) {
                        let mut sum = 0.0;
                        let mut n = 0.0;
                        for v in values {
                            if let Some(p) = v.get("percent").and_then(|x| x.as_f64()) {
                                sum += p;
                                n += 1.0;
                            }
                        }
                        if n > 0.0 { volume_percent = sum / n; }
                    }
                }
                let mute = i.get("mute").and_then(|x| x.as_bool()).unwrap_or(false);
                res.push(SinkInput{ index, sink, app_name, media_name, volume_percent, mute });
            }
        }
    }
    res
}

fn set_sink_input_volume(index: i64, percent: f64) {
    let pct = percent.round().clamp(0.0, 150.0) as i64;
    let _ = Command::new("pactl").args(["set-sink-input-volume", &index.to_string(), &format!("{}%", pct)]).status();
}

fn set_sink_input_mute(index: i64, mute: bool) {
    let _ = Command::new("pactl").args(["set-sink-input-mute", &index.to_string(), if mute {"1"} else {"0"}]).status();
}

fn move_sink_input(index: i64, sink_name_or_index: &str) {
    let _ = Command::new("pactl").args(["move-sink-input", &index.to_string(), sink_name_or_index]).status();
}

fn set_default_sink(name: &str) {
    let _ = Command::new("pactl").args(["set-default-sink", name]).status();
}

fn build_ui(app: &Application) -> ApplicationWindow {
    let win = ApplicationWindow::builder()
        .application(app)
        .title("EarTrumpet (Linux)")
        .default_width(900)
        .default_height(600)
        .build();

    let root = GtkBox::new(Orientation::Vertical, 8);

    let header = GtkBox::new(Orientation::Horizontal, 8);
    let refresh_btn = Button::with_label("Refresh");
    header.append(&refresh_btn);
    let default_sink_label = Label::new(Some("Default sink: loading…"));
    header.append(&default_sink_label);
    root.append(&header);

    let body = GtkBox::new(Orientation::Horizontal, 8);

    // Left: Sinks list
    let sinks_box = GtkBox::new(Orientation::Vertical, 6);
    sinks_box.append(&Label::new(Some("Devices (Sinks)")));
    let sinks_list = ListBox::new();
    let sinks_scroll = ScrolledWindow::builder().child(&sinks_list).min_content_height(200).build();
    sinks_box.append(&sinks_scroll);
    root.append(&sinks_box);

    // Right: Per-app streams
    let streams_box = GtkBox::new(Orientation::Vertical, 6);
    streams_box.append(&Label::new(Some("Per‑application streams")));
    let streams_list = ListBox::new();
    let streams_scroll = ScrolledWindow::builder().child(&streams_list).vexpand(true).hexpand(true).build();
    streams_box.append(&streams_scroll);
    body.append(&sinks_box);
    body.append(&streams_box);
    root.append(&body);

    win.set_child(Some(&root));

    let refresh_ui = clone!(@weak sinks_list, @weak streams_list, @weak default_sink_label => move || {
        // Update default sink label
        if let Some(def) = get_default_sink_name() {
            default_sink_label.set_text(&format!("Default sink: {}", def));
        } else {
            default_sink_label.set_text("Default sink: <unknown>");
        }

        // Populate sinks
        for child in sinks_list.children() { sinks_list.remove(&child); }
        let ss = sinks();
        for s in ss.iter() {
            let row = ListBoxRow::new();
            let row_box = GtkBox::new(Orientation::Horizontal, 6);
            let lbl = Label::new(Some(&format!("#{} {}", s.index, s.description)));
            lbl.set_xalign(0.0);
            row_box.append(&lbl);
            let set_default = Button::with_label("Set Default");
            let sink_name = s.name.clone();
            set_default.connect_clicked(clone!(@strong sink_name => move |_| { set_default_sink(&sink_name); }));
            row_box.append(&set_default);
            row.set_child(Some(&row_box));
            sinks_list.append(&row);
        }

        // Populate sink inputs
        for child in streams_list.children() { streams_list.remove(&child); }
        let sinputs = sink_inputs();
        let sinks_now = sinks();
        for si in sinputs.iter() {
            let row = ListBoxRow::new();
            let row_box = GtkBox::new(Orientation::Vertical, 4);
            let title = format!("#{}  {}  —  {}", si.index, si.app_name, si.media_name);
            row_box.append(&Label::new(Some(&title)));

            // Controls row
            let controls = GtkBox::new(Orientation::Horizontal, 6);

            // Volume slider
            let adj = Adjustment::new(si.volume_percent, 0.0, 150.0, 1.0, 10.0, 0.0);
            let slider = Scale::new(Orientation::Horizontal, Some(&adj));
            slider.set_draw_value(true);
            slider.set_hexpand(true);
            controls.append(&Label::new(Some("Volume:")));
            controls.append(&slider);

            // Mute toggle
            let mute_btn = Button::with_label(if si.mute { "Unmute" } else { "Mute" });
            let idx_mute = si.index;
            mute_btn.connect_clicked(move |b| {
                let to = b.label().map(|t| t.to_string()).unwrap_or_default() == "Mute";
                set_sink_input_mute(idx_mute, to);
            });
            controls.append(&mute_btn);

            // Move combobox
            let combo = ComboBoxText::new();
            for sk in sinks_now.iter() {
                combo.append_text(&format!("#{} {}", sk.index, sk.description));
                // store sink name in ID is trickier; we will move by index string
            }
            combo.set_active(Some(0));
            let idx_move = si.index;
            let sink_map: Vec<String> = sinks_now.iter().map(|sk| sk.index.to_string()).collect();
            combo.connect_changed(move |c| {
                if let Some(i) = c.active() {
                    if let Some(sink_idx_str) = sink_map.get(i as usize) {
                        move_sink_input(idx_move, sink_idx_str);
                    }
                }
            });
            controls.append(&Label::new(Some("Move to:")));
            controls.append(&combo);

            row_box.append(&controls);
            row.set_child(Some(&row_box));
            streams_list.append(&row);

            // Apply volume on slider change
            let idx_vol = si.index;
            slider.connect_value_changed(move |s| {
                let v = s.value();
                set_sink_input_volume(idx_vol, v);
            });
        }

        sinks_list.show_all();
        streams_list.show_all();
    });

    // Initial populate
    refresh_ui();

    // Refresh button
    refresh_btn.connect_clicked(clone!(@strong refresh_ui => move |_| { refresh_ui(); }));

    // Periodic refresh every 2 seconds
    timeout_add_local(Duration::from_secs(2), clone!(@strong refresh_ui => @default-return glib::ControlFlow::Continue, move || {
        refresh_ui();
        glib::ControlFlow::Continue
    }));

    win
}

fn main() {
    let app = Application::builder()
        .application_id("dev.ultimate_garuda.eartrumpet")
        .build();

    app.connect_activate(|app| {
        let win = build_ui(app);
        win.present();
    });

    app.run();
}
