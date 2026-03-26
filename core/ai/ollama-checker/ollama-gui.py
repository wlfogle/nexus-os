#!/usr/bin/env python3

import tkinter as tk
from tkinter import ttk, filedialog, messagebox, scrolledtext
import subprocess
import threading
import os
from pathlib import Path

class OllamaCodeCheckerGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("Ollama Code Checker")
        self.root.geometry("800x600")
        
        # Variables
        self.analysis_running = False
        self.current_process = None
        
        self.setup_ui()
        self.load_available_models()
    
    def setup_ui(self):
        # Main frame
        main_frame = ttk.Frame(self.root, padding="10")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Configure grid weights
        self.root.columnconfigure(0, weight=1)
        self.root.rowconfigure(0, weight=1)
        main_frame.columnconfigure(1, weight=1)
        main_frame.rowconfigure(5, weight=1)
        
        # Title
        title_label = ttk.Label(main_frame, text="Ollama Code Analysis Tool", 
                               font=('Arial', 16, 'bold'))
        title_label.grid(row=0, column=0, columnspan=3, pady=(0, 20))
        
        # Model selection
        ttk.Label(main_frame, text="Model:").grid(row=1, column=0, sticky=tk.W, pady=5)
        self.model_var = tk.StringVar(value="granite-code:latest")
        self.model_combo = ttk.Combobox(main_frame, textvariable=self.model_var, width=40)
        self.model_combo.grid(row=1, column=1, sticky=(tk.W, tk.E), pady=5, padx=(10, 0))
        
        # Analysis type
        ttk.Label(main_frame, text="Analysis Type:").grid(row=2, column=0, sticky=tk.W, pady=5)
        self.analysis_var = tk.StringVar(value="errors")
        analysis_frame = ttk.Frame(main_frame)
        analysis_frame.grid(row=2, column=1, sticky=(tk.W, tk.E), pady=5, padx=(10, 0))
        
        analysis_options = [
            ("Errors & Bugs", "errors"),
            ("Code Style", "style"),
            ("Security", "security"),
            ("Performance", "performance"),
            ("Cleanup (Stub/Unused)", "cleanup"),
            ("All", "all")
        ]
        
        for i, (text, value) in enumerate(analysis_options):
            ttk.Radiobutton(analysis_frame, text=text, variable=self.analysis_var, 
                           value=value).grid(row=0, column=i, padx=(0, 10))
        
        # Target selection
        ttk.Label(main_frame, text="Target:").grid(row=3, column=0, sticky=tk.W, pady=5)
        target_frame = ttk.Frame(main_frame)
        target_frame.grid(row=3, column=1, columnspan=2, sticky=(tk.W, tk.E), pady=5, padx=(10, 0))
        target_frame.columnconfigure(0, weight=1)
        
        self.target_var = tk.StringVar(value=str(Path.cwd()))
        self.target_entry = ttk.Entry(target_frame, textvariable=self.target_var)
        self.target_entry.grid(row=0, column=0, sticky=(tk.W, tk.E), padx=(0, 10))
        
        ttk.Button(target_frame, text="Browse Dir", 
                  command=self.browse_directory).grid(row=0, column=1)
        ttk.Button(target_frame, text="Browse File", 
                  command=self.browse_file).grid(row=0, column=2, padx=(5, 0))
        
        # Control buttons
        button_frame = ttk.Frame(main_frame)
        button_frame.grid(row=4, column=0, columnspan=3, pady=20)
        
        self.analyze_button = ttk.Button(button_frame, text="Start Analysis", 
                                        command=self.start_analysis)
        self.analyze_button.grid(row=0, column=0, padx=(0, 10))
        
        self.stop_button = ttk.Button(button_frame, text="Stop Analysis", 
                                     command=self.stop_analysis, state=tk.DISABLED)
        self.stop_button.grid(row=0, column=1, padx=(0, 10))
        
        ttk.Button(button_frame, text="Clear Output", 
                  command=self.clear_output).grid(row=0, column=2, padx=(0, 10))
        
        ttk.Button(button_frame, text="Save Report", 
                  command=self.save_report).grid(row=0, column=3)
        
        # Output area
        output_frame = ttk.LabelFrame(main_frame, text="Analysis Output", padding="5")
        output_frame.grid(row=5, column=0, columnspan=3, sticky=(tk.W, tk.E, tk.N, tk.S), pady=(10, 0))
        output_frame.columnconfigure(0, weight=1)
        output_frame.rowconfigure(0, weight=1)
        
        self.output_text = scrolledtext.ScrolledText(output_frame, wrap=tk.WORD, 
                                                    height=20, state=tk.DISABLED)
        self.output_text.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Progress bar
        self.progress = ttk.Progressbar(main_frame, mode='indeterminate')
        self.progress.grid(row=6, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(10, 0))
        
        # Status bar
        self.status_var = tk.StringVar(value="Ready")
        status_bar = ttk.Label(main_frame, textvariable=self.status_var, relief=tk.SUNKEN)
        status_bar.grid(row=7, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(5, 0))
    
    def load_available_models(self):
        """Load available Ollama models"""
        models_path = '/run/media/garuda/73cf9511-0af0-4ac4-9d83-ee21eb17ff5d/models'
        
        try:
            # Try to get models from manifest directory first (more reliable)
            manifest_path = f"{models_path}/manifests/registry.ollama.ai/library"
            if os.path.exists(manifest_path):
                model_dirs = [d for d in os.listdir(manifest_path) 
                             if os.path.isdir(os.path.join(manifest_path, d))]
                
                if model_dirs:
                    # Add :latest suffix to model names
                    available_models = [f"{model}:latest" for model in sorted(model_dirs)]
                    self.model_combo['values'] = available_models
                    
                    # Set default to a code model if available
                    code_models = [m for m in available_models if any(x in m.lower() 
                                  for x in ['code', 'granite', 'deepseek'])]
                    if code_models:
                        self.model_var.set(code_models[0])
                    else:
                        self.model_var.set(available_models[0])
                    return
            
            # Fallback: try ollama list command
            env = os.environ.copy()
            env['OLLAMA_MODELS'] = models_path
            
            result = subprocess.run(['ollama', 'list'], capture_output=True, text=True, env=env, timeout=10)
            if result.returncode == 0 and result.stdout.strip():
                lines = result.stdout.strip().split('\n')[1:]  # Skip header
                models = [line.split()[0] for line in lines if line.strip() and 'NAME' not in line]
                if models:
                    self.model_combo['values'] = models
                    code_models = [m for m in models if any(x in m.lower() 
                                  for x in ['code', 'granite', 'deepseek'])]
                    if code_models:
                        self.model_var.set(code_models[0])
                    return
                        
        except Exception as e:
            print(f"Warning: Could not load models list: {e}")
        
        # Final fallback: use common model names
        fallback_models = [
            'granite-code:latest', 'deepseek-coder-v2:latest', 'qwen2.5-coder:latest',
            'codellama:latest', 'codegemma:latest', 'llama3.1:latest'
        ]
        self.model_combo['values'] = fallback_models
        self.model_var.set('granite-code:latest')
    
    def browse_directory(self):
        """Browse for directory"""
        directory = filedialog.askdirectory(initialdir=self.target_var.get())
        if directory:
            self.target_var.set(directory)
    
    def browse_file(self):
        """Browse for file"""
        file_path = filedialog.askopenfilename(
            initialdir=os.path.dirname(self.target_var.get()),
            filetypes=[
                ("Code files", "*.py *.rs *.ts *.tsx *.js *.jsx *.go *.java *.cpp *.c *.h"),
                ("All files", "*.*")
            ]
        )
        if file_path:
            self.target_var.set(file_path)
    
    def append_output(self, text):
        """Append text to output area"""
        self.output_text.config(state=tk.NORMAL)
        self.output_text.insert(tk.END, text)
        self.output_text.see(tk.END)
        self.output_text.config(state=tk.DISABLED)
        self.root.update_idletasks()
    
    def clear_output(self):
        """Clear output area"""
        self.output_text.config(state=tk.NORMAL)
        self.output_text.delete(1.0, tk.END)
        self.output_text.config(state=tk.DISABLED)
    
    def save_report(self):
        """Save analysis report to file"""
        if not self.output_text.get(1.0, tk.END).strip():
            messagebox.showwarning("No Content", "No analysis output to save.")
            return
        
        file_path = filedialog.asksaveasfilename(
            defaultextension=".md",
            filetypes=[("Markdown files", "*.md"), ("Text files", "*.txt"), ("All files", "*.*")]
        )
        if file_path:
            try:
                with open(file_path, 'w') as f:
                    f.write(self.output_text.get(1.0, tk.END))
                messagebox.showinfo("Saved", f"Report saved to {file_path}")
            except Exception as e:
                messagebox.showerror("Error", f"Could not save file: {e}")
    
    def start_analysis(self):
        """Start code analysis"""
        if self.analysis_running:
            return
        
        target = self.target_var.get().strip()
        if not target:
            messagebox.showerror("Error", "Please select a target file or directory.")
            return
        
        if not os.path.exists(target):
            messagebox.showerror("Error", f"Target path does not exist: {target}")
            return
        
        self.analysis_running = True
        self.analyze_button.config(state=tk.DISABLED)
        self.stop_button.config(state=tk.NORMAL)
        self.progress.start()
        self.status_var.set("Running analysis...")
        
        # Clear previous output
        self.clear_output()
        
        # Run analysis in separate thread
        thread = threading.Thread(target=self.run_analysis)
        thread.daemon = True
        thread.start()
    
    def stop_analysis(self):
        """Stop running analysis"""
        if self.current_process:
            self.current_process.terminate()
        self.analysis_finished()
        self.append_output("\n--- Analysis stopped by user ---\n")
    
    def run_analysis(self):
        """Run the actual analysis"""
        try:
            target = self.target_var.get().strip()
            model = self.model_var.get()
            analysis_type = self.analysis_var.get()
            
            # Build command
            cmd = ['/home/garuda/ollama-code-checker.sh']
            cmd.extend(['-m', model])
            cmd.extend(['-t', analysis_type])
            
            if os.path.isfile(target):
                cmd.extend(['-f', target])
            else:
                cmd.append(target)
            
            self.append_output(f"Starting analysis...\n")
            self.append_output(f"Model: {model}\n")
            self.append_output(f"Type: {analysis_type}\n")
            self.append_output(f"Target: {target}\n")
            self.append_output("=" * 50 + "\n\n")
            
            # Run the analysis
            env = os.environ.copy()
            env['OLLAMA_MODELS'] = '/run/media/garuda/73cf9511-0af0-4ac4-9d83-ee21eb17ff5d/models'
            
            self.current_process = subprocess.Popen(
                cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, 
                text=True, bufsize=1, universal_newlines=True, env=env
            )
            
            # Read output line by line
            for line in iter(self.current_process.stdout.readline, ''):
                if line:
                    self.append_output(line)
            
            self.current_process.wait()
            
            if self.current_process.returncode == 0:
                self.append_output("\n--- Analysis completed successfully ---\n")
            else:
                self.append_output(f"\n--- Analysis finished with exit code {self.current_process.returncode} ---\n")
                
        except Exception as e:
            self.append_output(f"\nError during analysis: {e}\n")
        finally:
            self.root.after(0, self.analysis_finished)
    
    def analysis_finished(self):
        """Clean up after analysis finishes"""
        self.analysis_running = False
        self.current_process = None
        self.analyze_button.config(state=tk.NORMAL)
        self.stop_button.config(state=tk.DISABLED)
        self.progress.stop()
        self.status_var.set("Analysis complete")

def main():
    root = tk.Tk()
    app = OllamaCodeCheckerGUI(root)
    root.mainloop()

if __name__ == "__main__":
    main()