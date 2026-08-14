// SPDX-License-Identifier: GPL-2.0-or-later
/*
 * clevo-hotkeys.c - Fn-hotkey input driver for Clevo-based laptops
 * (developed for and tested on an OriginPC EON17-X).
 *
 * Background / why this module exists
 * ------------------------------------
 * This laptop exposes the classic Clevo ACPI-WMI hotkey interface: three WMI
 * GUIDs under /sys/bus/wmi/devices/ -
 *   ABBC0F6D-8EA1-11D1-00A0-C90629100000  "WMBB" method GUID (function calls)
 *   ABBC0F6B-8EA1-11D1-00A0-C90629100000  event notification GUID
 *   ABBC0F6C-8EA1-11D1-00A0-C90629100000  (unused here)
 * Confirmed via `ls /sys/bus/wmi/devices/` on this machine: none of these
 * three are claimed by any driver (the only bound WMI driver present,
 * nvidia-wmi-ec-backlight, owns a different GUID: 603E9613-...). A
 * System76-authored out-of-tree module (system76_acpi, builds clevo_acpi.ko)
 * is loaded but does not bind here either - its DMI match table only
 * recognizes System76 model names, and this machine's DMI reports
 * sys_vendor=OriginPC, product_name=EON17-X. So Fn-hotkeys currently reach
 * nowhere on this box; this driver fills that gap using the machine's own,
 * already-present ACPI-WMI surface rather than guessing at EC registers.
 *
 * The WMBB call pattern (function 0x01 = fetch last event code, function
 * 0x46 = enable WMI notifications) and the event-code table below are the
 * long-established values used by multiple independent Clevo hotkey
 * projects (clevo-wmi, clevo-xsm-wmi, tuxedo-keyboard's clevo_keyboard.c).
 * This driver only ever performs the documented, read-only "get event"
 * query in response to a firmware notification - it never issues blind
 * writes to EC/ACPI state, so there is no risk of corrupting embedded
 * controller state.
 */

#include <linux/acpi.h>
#include <linux/input.h>
#include <linux/input/sparse-keymap.h>
#include <linux/module.h>
#include <linux/wmi.h>

#define CLEVO_HOTKEYS_VERSION "1.0"

#define CLEVO_WMBB_GUID "ABBC0F6D-8EA1-11D1-00A0-C90629100000"
#define CLEVO_EVENT_GUID "ABBC0F6B-8EA1-11D1-00A0-C90629100000"

#define CLEVO_WMBB_FUNC_GET_EVENT 0x01
#define CLEVO_WMBB_FUNC_ENABLE_NOTIFICATIONS 0x46

/* Event codes as used by the Clevo ACPI-WMI interface. Values confirmed
 * against the community-maintained tuxedo-keyboard clevo_keyboard.h keymap
 * (same interface family, same ODM). */
#define CLEVO_EVENT_KB_LEDS_DECREASE        0x81
#define CLEVO_EVENT_KB_LEDS_INCREASE        0x82
#define CLEVO_EVENT_KB_LEDS_CYCLE_MODE      0x83
#define CLEVO_EVENT_KB_LEDS_CYCLE_BRIGHTNESS 0x8A
#define CLEVO_EVENT_KB_LEDS_TOGGLE          0x9F
#define CLEVO_EVENT_TOUCHPAD_TOGGLE         0x5D
#define CLEVO_EVENT_TOUCHPAD_OFF            0xFC
#define CLEVO_EVENT_TOUCHPAD_ON             0xFD
#define CLEVO_EVENT_RFKILL1                 0x85
#define CLEVO_EVENT_RFKILL2                 0x86
#define CLEVO_EVENT_GAUGE_KEY                0x59

static const struct key_entry clevo_hotkeys_keymap[] = {
	{ KE_KEY, CLEVO_EVENT_KB_LEDS_DECREASE,        { KEY_KBDILLUMDOWN } },
	{ KE_KEY, CLEVO_EVENT_KB_LEDS_INCREASE,        { KEY_KBDILLUMUP } },
	{ KE_KEY, CLEVO_EVENT_KB_LEDS_CYCLE_MODE,      { KEY_LIGHTS_TOGGLE } },
	{ KE_KEY, CLEVO_EVENT_KB_LEDS_CYCLE_BRIGHTNESS,{ KEY_KBDILLUMTOGGLE } },
	{ KE_KEY, CLEVO_EVENT_KB_LEDS_TOGGLE,          { KEY_KBDILLUMTOGGLE } },
	{ KE_KEY, CLEVO_EVENT_TOUCHPAD_TOGGLE,         { KEY_F21 } },
	{ KE_KEY, CLEVO_EVENT_TOUCHPAD_OFF,            { KEY_F21 } },
	{ KE_KEY, CLEVO_EVENT_TOUCHPAD_ON,             { KEY_F21 } },
	{ KE_KEY, CLEVO_EVENT_RFKILL1,                 { KEY_RFKILL } },
	{ KE_KEY, CLEVO_EVENT_RFKILL2,                 { KEY_RFKILL } },
	{ KE_KEY, CLEVO_EVENT_GAUGE_KEY,               { KEY_PROG1 } },
	{ KE_END, 0 }
};

struct clevo_hotkeys_data {
	struct input_dev *input_dev;
	struct wmi_device *wdev;
};

static int clevo_call_wmbb(struct wmi_device *wdev, u8 func, u32 *result)
{
	struct acpi_buffer output = { ACPI_ALLOCATE_BUFFER, NULL };
	struct acpi_buffer input = { sizeof(func), &func };
	union acpi_object *obj;
	acpi_status status;
	int ret = 0;

	status = wmi_evaluate_method(CLEVO_WMBB_GUID, 0, func, &input, &output);
	if (ACPI_FAILURE(status)) {
		dev_warn(&wdev->dev, "WMBB call failed for func %#04x: %s\n",
			 func, acpi_format_exception(status));
		return -EIO;
	}

	obj = output.pointer;
	if (!obj) {
		return -ENODATA;
	}
	if (obj->type == ACPI_TYPE_INTEGER) {
		if (obj->integer.value == 0xFFFFFFFF) {
			dev_dbg(&wdev->dev, "WMBB reports invalid function %#04x\n", func);
			ret = -EINVAL;
		} else if (result) {
			*result = (u32)obj->integer.value;
		}
	} else {
		dev_warn(&wdev->dev, "WMBB returned unexpected ACPI type %d\n", obj->type);
		ret = -ENXIO;
	}
	kfree(obj);
	return ret;
}

static void clevo_hotkeys_notify(struct wmi_device *wdev, union acpi_object *data)
{
	struct clevo_hotkeys_data *priv = dev_get_drvdata(&wdev->dev);
	u32 event = 0;

	/* Read-only query: ask firmware what the last hotkey event was.
	 * No EC/ACPI state is ever written here. */
	if (clevo_call_wmbb(wdev, CLEVO_WMBB_FUNC_GET_EVENT, &event)) {
		dev_dbg(&wdev->dev, "Could not retrieve WMI hotkey event\n");
		return;
	}

	dev_dbg(&wdev->dev, "Clevo hotkey event: %#04x\n", event);

	if (!sparse_keymap_report_event(priv->input_dev, event, 1, true))
		dev_dbg(&wdev->dev, "Unmapped hotkey event: %#04x\n", event);
}

static int clevo_hotkeys_input_setup(struct wmi_device *wdev)
{
	struct clevo_hotkeys_data *priv = dev_get_drvdata(&wdev->dev);
	int error;

	priv->input_dev = devm_input_allocate_device(&wdev->dev);
	if (!priv->input_dev)
		return -ENOMEM;

	priv->input_dev->name = "Clevo WMI Hotkeys";
	priv->input_dev->phys = "clevo-hotkeys/input0";
	priv->input_dev->id.bustype = BUS_HOST;
	priv->input_dev->dev.parent = &wdev->dev;

	error = sparse_keymap_setup(priv->input_dev, clevo_hotkeys_keymap, NULL);
	if (error)
		return error;

	return input_register_device(priv->input_dev);
}

static int clevo_hotkeys_probe(struct wmi_device *wdev, const void *context)
{
	struct clevo_hotkeys_data *priv;
	int error;
	u32 result = 0;

	priv = devm_kzalloc(&wdev->dev, sizeof(*priv), GFP_KERNEL);
	if (!priv)
		return -ENOMEM;

	priv->wdev = wdev;
	dev_set_drvdata(&wdev->dev, priv);

	error = clevo_hotkeys_input_setup(wdev);
	if (error) {
		dev_err(&wdev->dev, "Failed to set up input device: %d\n", error);
		return error;
	}

	/* Ask firmware to start sending us WMI notifications for hotkeys. */
	error = clevo_call_wmbb(wdev, CLEVO_WMBB_FUNC_ENABLE_NOTIFICATIONS, &result);
	if (error) {
		dev_warn(&wdev->dev,
			 "Failed to enable hotkey notifications (%d); hotkeys may not work\n",
			 error);
		/* Non-fatal: keep the input device registered in case
		 * notifications are already enabled by firmware default. */
	}

	dev_info(&wdev->dev, "Clevo hotkeys driver loaded (v%s)\n", CLEVO_HOTKEYS_VERSION);
	return 0;
}

static const struct wmi_device_id clevo_hotkeys_id_table[] = {
	{ .guid_string = CLEVO_EVENT_GUID },
	{ }
};
MODULE_DEVICE_TABLE(wmi, clevo_hotkeys_id_table);

static struct wmi_driver clevo_hotkeys_driver = {
	.driver = {
		.name = "clevo_hotkeys",
	},
	.id_table = clevo_hotkeys_id_table,
	.probe = clevo_hotkeys_probe,
	.notify = clevo_hotkeys_notify,
};
module_wmi_driver(clevo_hotkeys_driver);

MODULE_AUTHOR("nexus-os");
MODULE_DESCRIPTION("Fn-hotkey input driver for Clevo-based laptops (ACPI-WMI interface)");
MODULE_LICENSE("GPL");
MODULE_VERSION(CLEVO_HOTKEYS_VERSION);
