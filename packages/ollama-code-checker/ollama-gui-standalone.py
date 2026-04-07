#!/usr/bin/env python3

import tkinter as tk
from tkinter import ttk, filedialog, messagebox, scrolledtext
import subprocess
import threading
import os
from pathlib import Path
import json
import datetime

class OllamaCodeCheckerGUI:
    def __init__(self, root):
        self.root = root
        self.root.title("Ollama Code Checker")
        self.root.geometry("900x700")
        
        # Configuration
        self.models_path = "/run/media/garuda/73cf9511-0af0-4ac4-9d83-ee21eb17ff5d/models"
        
        # Variables
        self.analysis_running = False
        self.current_process = None
        self.analyzed_files = []  # Store files from last analysis
        self.last_analysis_target = None
        self.last_analysis_type = None
        self.git_info = {}  # Store git repository information
        
        self.setup_ui()
        self.load_available_models()
        self.start_ollama_service()
    
    def setup_ui(self):
        # Main frame
        main_frame = ttk.Frame(self.root, padding="10")
        main_frame.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Configure grid weights
        self.root.columnconfigure(0, weight=1)
        self.root.rowconfigure(0, weight=1)
        main_frame.columnconfigure(1, weight=1)
        main_frame.rowconfigure(6, weight=1)
        
        # Title
        title_label = ttk.Label(main_frame, text="🤖 Ollama Code Analysis Tool", 
                               font=('Arial', 18, 'bold'))
        title_label.grid(row=0, column=0, columnspan=3, pady=(0, 20))
        
        # Model selection
        ttk.Label(main_frame, text="AI Model:", font=('Arial', 10, 'bold')).grid(row=1, column=0, sticky=tk.W, pady=5)
        self.model_var = tk.StringVar(value="granite-code:latest")
        self.model_combo = ttk.Combobox(main_frame, textvariable=self.model_var, width=50, font=('Arial', 9))
        self.model_combo.grid(row=1, column=1, sticky=(tk.W, tk.E), pady=5, padx=(10, 0))
        
        # Analysis type
        ttk.Label(main_frame, text="Analysis Type:", font=('Arial', 10, 'bold')).grid(row=2, column=0, sticky=tk.W, pady=5)
        self.analysis_var = tk.StringVar(value="cleanup")
        analysis_frame = ttk.Frame(main_frame)
        analysis_frame.grid(row=2, column=1, sticky=(tk.W, tk.E), pady=5, padx=(10, 0))
        
        analysis_options = [
            ("🐛 Errors & Bugs", "errors"),
            ("🎨 Code Style", "style"),
            ("🔒 Security", "security"),
            ("⚡ Performance", "performance"),
            ("🧹 Cleanup (Stub/Unused)", "cleanup"),
            ("📋 All", "all")
        ]
        
        for i, (text, value) in enumerate(analysis_options):
            ttk.Radiobutton(analysis_frame, text=text, variable=self.analysis_var, 
                           value=value).grid(row=i//3, column=i%3, padx=(0, 15), pady=2, sticky=tk.W)
        
        # Target selection
        ttk.Label(main_frame, text="Target:", font=('Arial', 10, 'bold')).grid(row=3, column=0, sticky=tk.W, pady=5)
        target_frame = ttk.Frame(main_frame)
        target_frame.grid(row=3, column=1, columnspan=2, sticky=(tk.W, tk.E), pady=5, padx=(10, 0))
        target_frame.columnconfigure(0, weight=1)
        
        self.target_var = tk.StringVar(value="/home/garuda/nexus-terminal")
        self.target_entry = ttk.Entry(target_frame, textvariable=self.target_var, font=('Arial', 9))
        self.target_entry.grid(row=0, column=0, sticky=(tk.W, tk.E), padx=(0, 10))
        
        # Bind target change to auto-suggest model
        self.target_var.trace('w', self.on_target_change)
        
        ttk.Button(target_frame, text="📁 Browse Dir", 
                  command=self.browse_directory).grid(row=0, column=1)
        ttk.Button(target_frame, text="📄 Browse File", 
                  command=self.browse_file).grid(row=0, column=2, padx=(5, 0))
        ttk.Button(target_frame, text="🤖 Auto-Select Model", 
                  command=self.auto_select_model).grid(row=0, column=3, padx=(5, 0))
        
        # Git info section
        git_frame = ttk.LabelFrame(main_frame, text="📋 Repository Info", padding="5")
        git_frame.grid(row=4, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(10, 5))
        git_frame.columnconfigure(0, weight=1)
        git_frame.columnconfigure(1, weight=1)
        git_frame.columnconfigure(2, weight=1)
        git_frame.columnconfigure(3, weight=1)
        
        self.git_status_var = tk.StringVar(value="No repository selected")
        git_status_label = ttk.Label(git_frame, textvariable=self.git_status_var, 
                                    font=('Arial', 9), foreground='gray')
        git_status_label.grid(row=0, column=0, columnspan=3, sticky=(tk.W, tk.E))
        
        ttk.Button(git_frame, text="📁 Browse Repo", 
                  command=self.browse_repository).grid(row=1, column=0, padx=(0, 5))
        ttk.Button(git_frame, text="📊 Git Status", 
                  command=self.show_git_status).grid(row=1, column=1, padx=5)
        ttk.Button(git_frame, text="📝 Recent Changes", 
                  command=self.show_recent_changes).grid(row=1, column=2, padx=5)
        ttk.Button(git_frame, text="🌿 Branch Info", 
                  command=self.show_branch_info).grid(row=1, column=3, padx=(5, 0))
        
        # Control buttons
        button_frame = ttk.Frame(main_frame)
        button_frame.grid(row=5, column=0, columnspan=3, pady=20)
        
        self.analyze_button = ttk.Button(button_frame, text="🚀 Start Analysis", 
                                        command=self.start_analysis, 
                                        style='Accent.TButton')
        self.analyze_button.grid(row=0, column=0, padx=(0, 10))
        
        self.fix_button = ttk.Button(button_frame, text="🔧 Fix Issues", 
                                    command=self.start_fix_only, 
                                    state=tk.DISABLED,
                                    style='Accent.TButton')
        self.fix_button.grid(row=0, column=1, padx=(0, 10))
        
        self.autofix_button = ttk.Button(button_frame, text="⚡ Analyze & Auto-Fix", 
                                        command=self.start_autofix_analysis, 
                                        style='Accent.TButton')
        self.autofix_button.grid(row=0, column=2, padx=(0, 10))
        
        self.stop_button = ttk.Button(button_frame, text="⏹️ Stop Analysis", 
                                     command=self.stop_analysis, state=tk.DISABLED)
        self.stop_button.grid(row=0, column=3, padx=(0, 10))
        
        ttk.Button(button_frame, text="📂 Load Results", 
                  command=self.load_previous_results).grid(row=0, column=4, padx=(0, 10))
        
        ttk.Button(button_frame, text="🗑️ Clear Output", 
                  command=self.clear_output).grid(row=0, column=5, padx=(0, 10))
        
        ttk.Button(button_frame, text="💾 Save Report", 
                  command=self.save_report).grid(row=0, column=6)
        
        # Output area
        output_frame = ttk.LabelFrame(main_frame, text="📊 Analysis Output", padding="5")
        output_frame.grid(row=6, column=0, columnspan=3, sticky=(tk.W, tk.E, tk.N, tk.S), pady=(10, 0))
        output_frame.columnconfigure(0, weight=1)
        output_frame.rowconfigure(0, weight=1)
        
        self.output_text = scrolledtext.ScrolledText(output_frame, wrap=tk.WORD, 
                                                    height=25, state=tk.NORMAL,
                                                    font=('Consolas', 9))
        self.output_text.grid(row=0, column=0, sticky=(tk.W, tk.E, tk.N, tk.S))
        
        # Make text selectable but prevent editing except for copy operations
        self.output_text.bind('<Key>', self.on_key_press)
        # Allow right-click context menu for copy
        self.output_text.bind('<Button-3>', self.show_context_menu)
        
        # Progress bar
        self.progress = ttk.Progressbar(main_frame, mode='indeterminate')
        self.progress.grid(row=7, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(10, 0))
        
        # Status bar
        self.status_var = tk.StringVar(value="Ready - Select model and target, then click Start Analysis")
        status_bar = ttk.Label(main_frame, textvariable=self.status_var, relief=tk.SUNKEN, 
                              font=('Arial', 9))
        status_bar.grid(row=8, column=0, columnspan=3, sticky=(tk.W, tk.E), pady=(5, 0))
    
    def start_ollama_service(self):
        """Start Ollama service in background"""
        def start_service():
            try:
                # Kill any existing ollama processes
                subprocess.run(['pkill', '-f', 'ollama serve'], capture_output=True)
                
                # Start ollama with custom models path
                env = os.environ.copy()
                env['OLLAMA_MODELS'] = self.models_path
                
                subprocess.Popen(['ollama', 'serve'], 
                               stdout=subprocess.DEVNULL, 
                               stderr=subprocess.DEVNULL, 
                               env=env)
                
                self.root.after(0, lambda: self.status_var.set("Ollama service started - Ready for analysis"))
            except Exception as e:
                self.root.after(0, lambda: self.status_var.set(f"Warning: Could not start Ollama service: {e}"))
        
        threading.Thread(target=start_service, daemon=True).start()
    
    def load_available_models(self):
        """Load available Ollama models"""
        try:
            # Try to get models from manifest directory first
            manifest_path = f"{self.models_path}/manifests/registry.ollama.ai/library"
            if os.path.exists(manifest_path):
                model_dirs = [d for d in os.listdir(manifest_path) 
                             if os.path.isdir(os.path.join(manifest_path, d))]
                
                if model_dirs:
                    # Separate code models from others
                    code_models = []
                    other_models = []
                    
                    for model in sorted(model_dirs):
                        model_name = f"{model}:latest"
                        if any(x in model.lower() for x in ['code', 'granite', 'deepseek']):
                            code_models.append(f"🚀 {model_name}")
                        else:
                            other_models.append(model_name)
                    
                    # Combine with code models first
                    all_models = code_models + other_models
                    self.model_combo['values'] = all_models
                    
                    if code_models:
                        self.model_var.set(code_models[0])
                    return
        
        except Exception as e:
            print(f"Warning: Could not load models: {e}")
        
        # Fallback models
        fallback_models = [
            '🚀 granite-code:latest', '🚀 deepseek-coder-v2:latest', '🚀 qwen2.5-coder:latest',
            '🚀 codellama:latest', '🚀 codegemma:latest', 'llama3.1:latest'
        ]
        self.model_combo['values'] = fallback_models
        self.model_var.set('🚀 granite-code:latest')
    
    def browse_directory(self):
        """Browse for directory"""
        directory = filedialog.askdirectory(initialdir=self.target_var.get())
        if directory:
            self.target_var.set(directory)
            # Update git info and auto-suggest model when directory is selected
            self.update_git_info(directory)
            self.auto_select_model()
    
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
            # Update git info and auto-suggest model when file is selected
            self.update_git_info(os.path.dirname(file_path))
            self.auto_select_model()
    
    def browse_repository(self):
        """Browse specifically for git repositories"""
        directory = filedialog.askdirectory(
            title="Select Git Repository",
            initialdir=self.target_var.get() or os.path.expanduser("~")
        )
        if directory:
            # Check if it's a git repository
            git_root = self.find_git_root(directory)
            if git_root:
                self.target_var.set(git_root)
                self.update_git_info(git_root)
                self.auto_select_model()
                self.status_var.set(f"Repository selected: {os.path.basename(git_root)}")
            else:
                # Ask if user wants to select it anyway
                result = messagebox.askyesno(
                    "Not a Git Repository",
                    f"The selected directory is not a git repository.\n\n"
                    f"Do you want to select it anyway for analysis?"
                )
                if result:
                    self.target_var.set(directory)
                    self.git_status_var.set("❌ Not a git repository")
                    self.auto_select_model()
                else:
                    self.status_var.set("Repository selection cancelled")
    
    def on_key_press(self, event):
        """Handle key presses - allow copy operations but prevent editing"""
        # Allow Ctrl+C (copy) and Ctrl+A (select all)
        if event.state & 0x4:  # Ctrl key pressed
            if event.keysym.lower() in ['c', 'a']:
                return None  # Allow the operation
        
        # Allow arrow keys, Page Up/Down, Home, End for navigation
        if event.keysym in ['Up', 'Down', 'Left', 'Right', 'Prior', 'Next', 'Home', 'End']:
            return None
        
        # Block all other key presses to prevent editing
        return 'break'
    
    def show_context_menu(self, event):
        """Show context menu with copy option"""
        try:
            context_menu = tk.Menu(self.root, tearoff=0)
            context_menu.add_command(label="Copy", command=self.copy_selection)
            context_menu.add_command(label="Select All", command=self.select_all)
            context_menu.tk_popup(event.x_root, event.y_root)
        except:
            pass
    
    def copy_selection(self):
        """Copy selected text to clipboard"""
        try:
            self.root.clipboard_clear()
            selected_text = self.output_text.selection_get()
            self.root.clipboard_append(selected_text)
        except tk.TclError:
            pass  # No selection
    
    def select_all(self):
        """Select all text in output area"""
        self.output_text.tag_add(tk.SEL, "1.0", tk.END)
        self.output_text.mark_set(tk.INSERT, "1.0")
        self.output_text.see(tk.INSERT)
    
    def update_git_info(self, path):
        """Update git repository information"""
        if not path or not os.path.exists(path):
            return
        
        # Find git root
        git_root = self.find_git_root(path)
        if not git_root:
            self.git_status_var.set("❌ Not a git repository")
            self.git_info = {}
            return
        
        try:
            # Get basic git info
            branch = self.run_git_command(git_root, ['branch', '--show-current']).strip()
            status = self.run_git_command(git_root, ['status', '--porcelain'])
            
            # Count changes
            modified_files = len([line for line in status.split('\n') if line.strip()])
            
            # Get last commit info
            try:
                last_commit = self.run_git_command(git_root, ['log', '-1', '--pretty=format:%h - %s (%cr)'])
            except:
                last_commit = "No commits"
            
            self.git_info = {
                'root': git_root,
                'branch': branch or 'detached HEAD',
                'status': status,
                'modified_files': modified_files,
                'last_commit': last_commit
            }
            
            # Update status display
            status_text = f"🌿 Branch: {self.git_info['branch']}"
            if modified_files > 0:
                status_text += f" | ⚠️ {modified_files} changes"
            else:
                status_text += f" | ✅ Clean"
            
            self.git_status_var.set(status_text)
            
        except Exception as e:
            self.git_status_var.set(f"❌ Git error: {str(e)}")
            self.git_info = {}
    
    def find_git_root(self, path):
        """Find the root of a git repository"""
        current_path = Path(path).resolve()
        
        while current_path != current_path.parent:
            if (current_path / '.git').exists():
                return str(current_path)
            current_path = current_path.parent
        
        return None
    
    def run_git_command(self, cwd, args):
        """Run a git command in the specified directory"""
        result = subprocess.run(
            ['git'] + args,
            cwd=cwd,
            capture_output=True,
            text=True
        )
        if result.returncode != 0:
            raise Exception(f"Git command failed: {result.stderr}")
        return result.stdout
    
    def show_git_status(self):
        """Show detailed git status"""
        if not self.git_info:
            messagebox.showinfo("Git Status", "No git repository information available.")
            return
        
        try:
            # Get detailed status
            status = self.run_git_command(self.git_info['root'], ['status', '--porcelain', '-v'])
            branch_info = self.run_git_command(self.git_info['root'], ['branch', '-vv'])
            
            self.clear_output()
            self.append_output("📊 Git Repository Status\n")
            self.append_output("=" * 50 + "\n")
            self.append_output(f"📁 Repository: {os.path.basename(self.git_info['root'])}\n")
            self.append_output(f"🌿 Current Branch: {self.git_info['branch']}\n")
            self.append_output(f"📝 Last Commit: {self.git_info['last_commit']}\n\n")
            
            if status.strip():
                self.append_output("📋 File Changes:\n")
                self.append_output("-" * 30 + "\n")
                for line in status.strip().split('\n'):
                    if line.strip():
                        status_code = line[:2]
                        filename = line[3:]
                        emoji = self.get_status_emoji(status_code)
                        self.append_output(f"{emoji} {status_code} {filename}\n")
            else:
                self.append_output("✅ Working directory clean\n")
            
            self.append_output("\n" + "=" * 50 + "\n")
            
        except Exception as e:
            messagebox.showerror("Git Error", f"Failed to get git status: {str(e)}")
    
    def show_recent_changes(self):
        """Show recent git commits and changes"""
        if not self.git_info:
            messagebox.showinfo("Recent Changes", "No git repository information available.")
            return
        
        try:
            # Get recent commits
            commits = self.run_git_command(self.git_info['root'], 
                ['log', '--oneline', '-10', '--pretty=format:%h|%cr|%s'])
            
            # Get recently changed files
            changed_files = self.run_git_command(self.git_info['root'], 
                ['diff', '--name-status', 'HEAD~5..HEAD'])
            
            self.clear_output()
            self.append_output("📝 Recent Changes Analysis\n")
            self.append_output("=" * 50 + "\n\n")
            
            self.append_output("🕒 Recent Commits (Last 10):\n")
            self.append_output("-" * 40 + "\n")
            for commit in commits.strip().split('\n'):
                if '|' in commit:
                    hash_part, time_part, message = commit.split('|', 2)
                    self.append_output(f"• {hash_part} ({time_part}) - {message}\n")
            
            if changed_files.strip():
                self.append_output("\n📁 Files Changed in Last 5 Commits:\n")
                self.append_output("-" * 40 + "\n")
                for line in changed_files.strip().split('\n'):
                    if line.strip():
                        parts = line.split('\t')
                        if len(parts) >= 2:
                            status, filename = parts[0], parts[1]
                            emoji = self.get_status_emoji(status)
                            self.append_output(f"{emoji} {status} {filename}\n")
            
            self.append_output("\n" + "=" * 50 + "\n")
            
        except Exception as e:
            messagebox.showerror("Git Error", f"Failed to get recent changes: {str(e)}")
    
    def show_branch_info(self):
        """Show git branch information and repository stats"""
        if not self.git_info:
            messagebox.showinfo("Branch Info", "No git repository information available.")
            return
        
        try:
            # Get branch info
            branches = self.run_git_command(self.git_info['root'], ['branch', '-a'])
            remotes = self.run_git_command(self.git_info['root'], ['remote', '-v'])
            
            # Get repository stats
            total_commits = self.run_git_command(self.git_info['root'], ['rev-list', '--count', 'HEAD']).strip()
            contributors = self.run_git_command(self.git_info['root'], 
                ['shortlog', '-sn', '--all']).strip().split('\n')[:5]
            
            self.clear_output()
            self.append_output("🌿 Git Branch & Repository Info\n")
            self.append_output("=" * 50 + "\n\n")
            
            self.append_output(f"📊 Repository Stats:\n")
            self.append_output(f"• Total Commits: {total_commits}\n")
            self.append_output(f"• Current Branch: {self.git_info['branch']}\n")
            self.append_output(f"• Repository Root: {self.git_info['root']}\n\n")
            
            self.append_output("🌿 All Branches:\n")
            self.append_output("-" * 30 + "\n")
            for branch in branches.strip().split('\n'):
                if branch.strip():
                    if branch.startswith('* '):
                        self.append_output(f"→ {branch[2:]} (current)\n")
                    else:
                        self.append_output(f"  {branch.strip()}\n")
            
            if remotes.strip():
                self.append_output("\n🔗 Remotes:\n")
                self.append_output("-" * 30 + "\n")
                for remote in remotes.strip().split('\n'):
                    self.append_output(f"• {remote}\n")
            
            if contributors:
                self.append_output("\n👥 Top Contributors:\n")
                self.append_output("-" * 30 + "\n")
                for contributor in contributors[:5]:
                    if contributor.strip():
                        self.append_output(f"• {contributor}\n")
            
            self.append_output("\n" + "=" * 50 + "\n")
            
        except Exception as e:
            messagebox.showerror("Git Error", f"Failed to get branch info: {str(e)}")
    
    def get_status_emoji(self, status_code):
        """Get emoji for git status codes"""
        status_map = {
            'M': '📝',   # Modified
            'A': '➕',   # Added
            'D': '❌',   # Deleted
            'R': '🔄',   # Renamed
            'C': '📋',   # Copied
            'U': '⚠️',   # Unmerged
            '?': '❓',   # Untracked
            '!': '🚫'    # Ignored
        }
        return status_map.get(status_code.strip(), '📄')
    
    def append_output(self, text):
        """Append text to output area"""
        # Text is always in NORMAL state for selection, just insert and scroll
        self.output_text.insert(tk.END, text)
        self.output_text.see(tk.END)
        self.root.update_idletasks()
    
    def clear_output(self):
        """Clear output area"""
        # Text is always in NORMAL state, just delete content
        self.output_text.delete(1.0, tk.END)
    
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
                self.status_var.set(f"Report saved: {file_path}")
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
        
        # Clean model name (remove emoji prefix)
        model = self.model_var.get().replace('🚀 ', '').strip()
        
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
        self.append_output("\n🛑 Analysis stopped by user\n")
    
    def start_autofix_analysis(self):
        """Start analysis with auto-fix capability"""
        if self.analysis_running:
            return
        
        target = self.target_var.get().strip()
        if not target:
            messagebox.showerror("Error", "Please select a target file or directory.")
            return
        
        if not os.path.exists(target):
            messagebox.showerror("Error", f"Target path does not exist: {target}")
            return
        
        # Confirm auto-fix action
        result = messagebox.askyesno(
            "Auto-Fix Confirmation", 
            "This will analyze and automatically fix issues in your code.\n\n"
            "⚠️ IMPORTANT: This will modify your files!\n\n"
            "Make sure you have backups or are using version control.\n\n"
            "Do you want to continue?"
        )
        
        if not result:
            return
        
        # Clean model name (remove emoji prefix)
        model = self.model_var.get().replace('🚀 ', '').strip()
        
        self.analysis_running = True
        self.analyze_button.config(state=tk.DISABLED)
        self.autofix_button.config(state=tk.DISABLED)
        self.stop_button.config(state=tk.NORMAL)
        self.progress.start()
        self.status_var.set("Running analysis with auto-fix...")
        
        # Clear previous output
        self.clear_output()
        
        # Run analysis in separate thread
        thread = threading.Thread(target=self.run_autofix_analysis)
        thread.daemon = True
        thread.start()
    
    def run_analysis(self):
        """Run the actual analysis using direct ollama commands"""
        try:
            target = self.target_var.get().strip()
            model = self.model_var.get().replace('🚀 ', '').strip()
            analysis_type = self.analysis_var.get()
            
            self.append_output("🚀 Starting Ollama Code Analysis\n")
            self.append_output("=" * 50 + "\n")
            self.append_output(f"📁 Target: {target}\n")
            self.append_output(f"🤖 Model: {model}\n")
            self.append_output(f"🔍 Analysis: {analysis_type}\n")
            self.append_output("=" * 50 + "\n\n")
            
            # Set environment
            env = os.environ.copy()
            env['OLLAMA_MODELS'] = self.models_path
            env['CUDA_VISIBLE_DEVICES'] = '0'  # Use GPU
            
            if os.path.isfile(target):
                files_to_analyze = [target]
            else:
                # Find code files in directory
                extensions = ['.rs', '.ts', '.tsx', '.js', '.jsx', '.py', '.go', '.java', '.cpp', '.c', '.h']
                files_to_analyze = []
                for root, dirs, files in os.walk(target):
                    # Skip common build/dependency directories
                    dirs[:] = [d for d in dirs if d not in ['node_modules', 'target', 'build', 'dist', '.git', '__pycache__']]
                    for file in files:
                        if any(file.endswith(ext) for ext in extensions):
                            file_path = os.path.join(root, file)
                            # Skip very large files (>100KB for better analysis)
                            try:
                                if os.path.getsize(file_path) < 100000:  # 100KB limit
                                    files_to_analyze.append(file_path)
                            except OSError:
                                pass  # Skip if can't get file size
            
            if not files_to_analyze:
                self.append_output("⚠️ No code files found to analyze.\n")
                return
            
            # Show warning for large codebases
            if len(files_to_analyze) > 100:
                self.append_output(f"⚠️  Large codebase detected: {len(files_to_analyze)} files\n")
                self.append_output("This may take a while. Consider analyzing smaller directories first.\n\n")
            else:
                self.append_output(f"📊 Found {len(files_to_analyze)} files to analyze\n\n")
            
            for i, file_path in enumerate(files_to_analyze, 1):
                if not self.analysis_running:
                    break
                    
                self.append_output(f"[{i}/{len(files_to_analyze)}] 🔍 Analyzing: {os.path.basename(file_path)}\n")
                
                # Read file content
                try:
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                        content = f.read()[:15000]  # Increase limit for better analysis
                    
                    if not content.strip():
                        self.append_output(f"⚠️ Skipping empty file: {os.path.basename(file_path)}\n\n")
                        continue
                        
                except Exception as e:
                    self.append_output(f"❌ Error reading file: {e}\n\n")
                    continue
                
                # Create analysis prompt
                prompt = self.create_analysis_prompt(file_path, content, analysis_type)
                
                # Debug: Show content size
                self.append_output(f"   Content size: {len(content)} chars\n")
                
                # Run Ollama analysis
                try:
                    self.current_process = subprocess.Popen(
                        ['ollama', 'run', model],
                        stdin=subprocess.PIPE,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.STDOUT,
                        text=True,
                        env=env
                    )
                    
                    stdout, _ = self.current_process.communicate(input=prompt, timeout=300)
                    
                    if stdout:
                        # Clean up terminal control sequences
                        clean_output = self.clean_terminal_output(stdout)
                        
                        # Check for common unhelpful responses
                        unhelpful_phrases = [
                            "i don't have access",
                            "i cannot access",
                            "i'm unable to see",
                            "i can't see the code",
                            "no code provided",
                            "code is not provided",
                            "i need to see the code",
                            "i need more information",
                            "please provide",
                            "provide me with",
                            "i'd be happy to help",
                            "sure, i can analyze",
                            "please provide the file",
                            "provide more context",
                            "it appears to be incomplete",
                            "cannot provide a detailed analysis",
                            "// your comprehensive code analysis goes here"
                        ]
                        
                        is_unhelpful = any(phrase in clean_output.lower() for phrase in unhelpful_phrases)
                        
                        if clean_output.strip() and not is_unhelpful:
                            self.append_output(f"✅ Results for {os.path.basename(file_path)}:\n")
                            self.append_output("-" * 40 + "\n")
                            self.append_output(clean_output)
                            self.append_output("\n" + "=" * 50 + "\n\n")
                        elif is_unhelpful:
                            self.append_output(f"🔄 Retrying analysis for {os.path.basename(file_path)} with simplified prompt...\n")
                            # Retry with a more direct prompt
                            file_ext = os.path.splitext(file_path)[1]
                            lang_name = {
                                '.py': 'Python', '.cpp': 'C++', '.h': 'C++', '.c': 'C',
                                '.js': 'JavaScript', '.ts': 'TypeScript', '.rs': 'Rust',
                                '.go': 'Go', '.java': 'Java'
                            }.get(file_ext, 'code')
                            
                            retry_prompt = f"""Here is {lang_name} code to analyze:

{content}

Analyze this code and identify any:
- Errors or bugs
- Style issues
- Security problems
- Performance issues
- Code that needs cleanup

Be specific about what you find."""
                            try:
                                retry_process = subprocess.Popen(
                                    ['ollama', 'run', model],
                                    stdin=subprocess.PIPE,
                                    stdout=subprocess.PIPE,
                                    stderr=subprocess.STDOUT,
                                    text=True,
                                    env=env
                                )
                                retry_stdout, _ = retry_process.communicate(input=retry_prompt, timeout=300)
                                if retry_stdout:
                                    retry_output = self.clean_terminal_output(retry_stdout)
                                    if retry_output.strip():
                                        self.append_output(f"✅ Results for {os.path.basename(file_path)} (retry):\n")
                                        self.append_output("-" * 40 + "\n")
                                        self.append_output(retry_output)
                                        self.append_output("\n" + "=" * 50 + "\n\n")
                                    else:
                                        self.append_output(f"⚠️ No useful output after retry for {os.path.basename(file_path)}\n\n")
                                else:
                                    self.append_output(f"⚠️ No output after retry for {os.path.basename(file_path)}\n\n")
                            except Exception as retry_e:
                                self.append_output(f"❌ Retry failed for {os.path.basename(file_path)}: {retry_e}\n\n")
                        else:
                            self.append_output(f"⚠️ No readable output from analysis of {os.path.basename(file_path)}\n\n")
                    
                except subprocess.TimeoutExpired:
                    self.append_output("⏱️ Analysis timed out for this file\n\n")
                    if self.current_process:
                        self.current_process.kill()
                except Exception as e:
                    self.append_output(f"❌ Error analyzing file: {e}\n\n")
            
            # Store analyzed files for potential fixing
            self.analyzed_files = files_to_analyze
            self.last_analysis_target = target
            self.last_analysis_type = analysis_type
            
            self.append_output("🎉 Analysis completed successfully!\n")
            self.append_output(f"📋 {len(files_to_analyze)} files analyzed and ready for fixing.\n")
            
        except Exception as e:
            self.append_output(f"\n❌ Analysis error: {e}\n")
        finally:
            self.root.after(0, self.analysis_finished)
    
    def create_analysis_prompt(self, file_path, content, analysis_type):
        """Create analysis prompt based on type"""
        file_ext = os.path.splitext(file_path)[1]
        language_map = {
            '.rs': 'Rust', '.ts': 'TypeScript', '.tsx': 'TypeScript', '.js': 'JavaScript', 
            '.jsx': 'JavaScript', '.py': 'Python', '.go': 'Go', '.java': 'Java',
            '.cpp': 'C++', '.c': 'C', '.h': 'C/C++'
        }
        language = language_map.get(file_ext, 'Unknown')
        
        # Create a much more direct and commanding prompt
        base_prompt = f"""ANALYZE THIS {language.upper()} CODE:

{content}

You must analyze the code above. Do not ask for more information - the code is right there. Analyze it now."""
        
        if analysis_type == "cleanup":
            return base_prompt + """TASK: Code Cleanup Analysis

Please analyze the code above and identify:
1. Stub functions (empty or placeholder implementations)
2. Unused functions, variables, and imports
3. Dead code that's never called
4. Commented-out code that should be removed
5. Redundant or duplicate code
6. Empty catch blocks or TODO comments

For each finding, specify:
- Line numbers where the issue occurs
- Description of the problem
- Whether it's safe to remove

Provide your analysis in a clear, structured format."""
            
        elif analysis_type == "errors":
            return base_prompt + """TASK: Error and Bug Detection

Please analyze the code above and identify:
1. Syntax errors
2. Type errors
3. Logic issues
4. Potential runtime errors
5. Missing imports or dependencies

For each issue found, provide:
- Line numbers where the error occurs
- Description of the problem
- Suggested fix

Provide your analysis in a clear, structured format."""
            
        elif analysis_type == "security":
            return base_prompt + """TASK: Security Analysis

Please analyze the code above and identify:
1. Security vulnerabilities
2. Unsafe operations
3. Input validation issues
4. Authentication problems
5. Data exposure risks

For each security issue found, provide:
- Line numbers where the vulnerability occurs
- Description of the security risk
- Severity level (High/Medium/Low)
- Recommended fix

Provide your analysis in a clear, structured format."""
            
        elif analysis_type == "performance":
            return base_prompt + """TASK: Performance Analysis

Please analyze the code above and identify:
1. Performance bottlenecks
2. Inefficient algorithms
3. Memory usage issues
4. I/O optimization opportunities

For each performance issue found, provide:
- Line numbers where the issue occurs
- Description of the performance problem
- Impact level (High/Medium/Low)
- Suggested optimization

Provide your analysis in a clear, structured format."""
            
        elif analysis_type == "style":
            return base_prompt + """TASK: Code Style Review

Please review the code above for:
1. Code formatting and indentation
2. Naming conventions
3. Code organization
4. Documentation quality
5. Best practice compliance

For each style issue found, provide:
- Line numbers where the issue occurs
- Description of the style problem
- Recommended improvement

Provide your analysis in a clear, structured format."""
            
        else:  # all
            return base_prompt + """TASK: Comprehensive Code Analysis

Please provide a thorough analysis of the code above covering:
1. Errors and bugs
2. Code style and best practices
3. Security considerations
4. Performance opportunities
5. Code cleanup (stub/unused code)

For each issue found, provide:
- Category (Error/Style/Security/Performance/Cleanup)
- Line numbers where the issue occurs
- Description of the problem
- Recommended fix or improvement
- Priority level (High/Medium/Low)

Provide your analysis in a clear, structured format with specific, actionable feedback."""
    
    def run_autofix_analysis(self):
        """Run analysis with auto-fix capability"""
        try:
            target = self.target_var.get().strip()
            model = self.model_var.get().replace('🚀 ', '').strip()
            analysis_type = self.analysis_var.get()
            
            self.append_output("🔧 Starting Ollama Auto-Fix Analysis\n")
            self.append_output("=" * 50 + "\n")
            self.append_output(f"📁 Target: {target}\n")
            self.append_output(f"🤖 Model: {model}\n")
            self.append_output(f"🔍 Analysis: {analysis_type}\n")
            self.append_output("⚠️  Auto-fix mode: Files will be modified!\n")
            self.append_output("=" * 50 + "\n\n")
            
            # Set environment
            env = os.environ.copy()
            env['OLLAMA_MODELS'] = self.models_path
            env['CUDA_VISIBLE_DEVICES'] = '0'  # Use GPU
            
            if os.path.isfile(target):
                files_to_analyze = [target]
            else:
                # Find code files in directory
                extensions = ['.rs', '.ts', '.tsx', '.js', '.jsx', '.py', '.go', '.java', '.cpp', '.c', '.h']
                files_to_analyze = []
                for root, dirs, files in os.walk(target):
                    # Skip common build/dependency directories
                    dirs[:] = [d for d in dirs if d not in ['node_modules', 'target', 'build', 'dist', '.git', '__pycache__']]
                    for file in files:
                        if any(file.endswith(ext) for ext in extensions):
                            file_path = os.path.join(root, file)
                            # Skip very large files (>50KB) for safety in auto-fix
                            try:
                                if os.path.getsize(file_path) < 50000:  # 50KB limit
                                    files_to_analyze.append(file_path)
                            except OSError:
                                pass  # Skip if can't get file size
            
            if not files_to_analyze:
                self.append_output("⚠️ No code files found to analyze.\n")
                return
            
            # Additional confirmation for large codebases in auto-fix mode
            if len(files_to_analyze) > 50:
                large_result = messagebox.askyesno(
                    "Large Codebase Warning", 
                    f"Found {len(files_to_analyze)} files to analyze and fix.\n\n"
                    f"This is a large codebase and processing may take a long time.\n\n"
                    f"⚠️ ALL {len(files_to_analyze)} FILES WILL BE MODIFIED!\n\n"
                    f"Are you sure you want to continue?"
                )
                if not large_result:
                    self.append_output("🛑 Auto-fix cancelled by user\n")
                    return
            
            self.append_output(f"📈 Found {len(files_to_analyze)} files to analyze and fix\n\n")
            
            fixed_files = 0
            
            for i, file_path in enumerate(files_to_analyze, 1):
                if not self.analysis_running:
                    break
                    
                self.append_output(f"[{i}/{len(files_to_analyze)}] 🔍 Analyzing: {os.path.basename(file_path)}\n")
                
                try:
                    # Read original file content
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                        original_content = f.read()
                    
                    if len(original_content) > 10000:  # Skip very large files
                        self.append_output(f"⚠️ Skipping large file (>10k chars): {os.path.basename(file_path)}\n\n")
                        continue
                    
                    # Create fix prompt
                    prompt = self.create_fix_prompt(file_path, original_content, analysis_type)
                    
                    # Get fixed code from Ollama
                    self.current_process = subprocess.Popen(
                        ['ollama', 'run', model],
                        stdin=subprocess.PIPE,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.STDOUT,
                        text=True,
                        env=env
                    )
                    
                    stdout, _ = self.current_process.communicate(input=prompt, timeout=300)
                    
                    if stdout:
                        # Clean up terminal control sequences first
                        clean_stdout = self.clean_terminal_output(stdout)
                        # Extract fixed code from AI response
                        fixed_code = self.extract_code_from_response(clean_stdout, original_content)
                        
                        if fixed_code and fixed_code != original_content:
                            # Create backup
                            backup_path = file_path + '.backup'
                            with open(backup_path, 'w', encoding='utf-8') as f:
                                f.write(original_content)
                            
                            # Write fixed code
                            with open(file_path, 'w', encoding='utf-8') as f:
                                f.write(fixed_code)
                            
                            self.append_output(f"✅ Fixed: {os.path.basename(file_path)} (backup created)\n")
                            fixed_files += 1
                        else:
                            self.append_output(f"ℹ️ No fixes needed: {os.path.basename(file_path)}\n")
                    
                except subprocess.TimeoutExpired:
                    self.append_output(f"⏱️ Timeout analyzing: {os.path.basename(file_path)}\n")
                    if self.current_process:
                        self.current_process.kill()
                except Exception as e:
                    self.append_output(f"❌ Error processing {os.path.basename(file_path)}: {str(e)}\n")
                
                self.append_output("\n")
            
            self.append_output("=" * 50 + "\n")
            self.append_output(f"🎉 Auto-fix complete! Fixed {fixed_files}/{len(files_to_analyze)} files\n")
            self.append_output("💾 Backup files created with .backup extension\n")
            
        except Exception as e:
            self.append_output(f"\n❌ Auto-fix error: {e}\n")
        finally:
            self.root.after(0, self.analysis_finished)
    
    def create_fix_prompt(self, file_path, content, analysis_type):
        """Create prompt for fixing code"""
        file_ext = os.path.splitext(file_path)[1]
        language_map = {
            '.rs': 'Rust', '.ts': 'TypeScript', '.tsx': 'TypeScript', '.js': 'JavaScript', 
            '.jsx': 'JavaScript', '.py': 'Python', '.go': 'Go', '.java': 'Java',
            '.cpp': 'C++', '.c': 'C', '.h': 'C/C++'
        }
        language = language_map.get(file_ext, 'Unknown')
        
        base_prompt = f"""You are an expert {language} programmer. Fix the issues in this code file.

File: {os.path.basename(file_path)}
Language: {language}

ORIGINAL CODE TO FIX:
```{language.lower()}
{content}
```

Task: Analyze the code above and fix any issues you find:
1. Fix errors, bugs, or issues
2. Remove unused imports, variables, and functions
3. Remove commented-out code
4. Fix code style issues
5. Improve performance where possible

IMPORTANT: Return ONLY the corrected code wrapped in ```{language.lower()} code blocks. Do not include explanations or other text. The code is provided above - analyze it directly."""
        
        return base_prompt
    
    def clean_terminal_output(self, text):
        """Remove terminal control sequences and formatting codes from text"""
        import re
        
        # Remove ANSI escape sequences (colors, cursor movement, etc.)
        ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
        text = ansi_escape.sub('', text)
        
        # Remove specific terminal control sequences
        control_sequences = [
            r'\[\?\d+[hl]',  # Private mode sequences
            r'\[\d*[ABCD]',  # Cursor movement
            r'\[\d*[GKJ]',   # Erase sequences
            r'\[\d+;\d+[Hf]', # Cursor position
            r'\[\?25[lh]',   # Cursor visibility
            r'\[\?2026[hl]', # Bracketed paste mode
            r'\[2K',         # Clear line
            r'\[1G',         # Move to column 1
            r'\[K',          # Clear to end of line
        ]
        
        for pattern in control_sequences:
            text = re.sub(pattern, '', text)
        
        # Remove spinner characters and extra whitespace
        spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
        for char in spinner_chars:
            text = text.replace(char, '')
        
        # Clean up excessive whitespace and newlines
        text = re.sub(r'\n\s*\n\s*\n', '\n\n', text)  # Multiple newlines to double
        text = re.sub(r'[ \t]+', ' ', text)  # Multiple spaces to single
        text = text.strip()
        
        return text
    
    def detect_dominant_language(self, target_path):
        """Detect the dominant programming language in target"""
        if not os.path.exists(target_path):
            return None
        
        language_counts = {}
        extensions = ['.rs', '.ts', '.tsx', '.js', '.jsx', '.py', '.go', '.java', '.cpp', '.c', '.h']
        
        if os.path.isfile(target_path):
            ext = os.path.splitext(target_path)[1]
            if ext in extensions:
                return self.get_language_from_extension(ext)
            return None
        
        # Count files by language
        for root, dirs, files in os.walk(target_path):
            dirs[:] = [d for d in dirs if d not in ['node_modules', 'target', 'build', 'dist', '.git']]
            for file in files:
                ext = os.path.splitext(file)[1]
                if ext in extensions:
                    lang = self.get_language_from_extension(ext)
                    language_counts[lang] = language_counts.get(lang, 0) + 1
        
        if not language_counts:
            return None
        
        # Return most common language
        return max(language_counts, key=language_counts.get)
    
    def get_language_from_extension(self, ext):
        """Map file extension to language name"""
        ext_map = {
            '.rs': 'rust',
            '.ts': 'typescript', '.tsx': 'typescript',
            '.js': 'javascript', '.jsx': 'javascript',
            '.py': 'python',
            '.go': 'go',
            '.java': 'java',
            '.cpp': 'cpp', '.c': 'c', '.h': 'c'
        }
        return ext_map.get(ext, 'general')
    
    def get_best_model_for_language(self, language):
        """Get the best available model for a specific language"""
        available_models = self.model_combo['values']
        
        # Model preferences by language
        language_models = {
            'rust': ['granite-code', 'deepseek-coder', 'codellama'],
            'typescript': ['granite-code', 'deepseek-coder', 'codegemma'],
            'javascript': ['granite-code', 'deepseek-coder', 'codegemma'],
            'python': ['deepseek-coder', 'granite-code', 'codellama'],
            'go': ['granite-code', 'deepseek-coder', 'codellama'],
            'java': ['granite-code', 'deepseek-coder', 'codellama'],
            'cpp': ['granite-code', 'deepseek-coder', 'codellama'],
            'c': ['granite-code', 'deepseek-coder', 'codellama']
        }
        
        preferred_models = language_models.get(language, ['granite-code', 'deepseek-coder'])
        
        # Find best available model
        for preferred in preferred_models:
            for model in available_models:
                if preferred in model.lower():
                    return model
        
        # Fallback to first code model
        for model in available_models:
            if '🚀' in model:
                return model
        
        return available_models[0] if available_models else 'granite-code:latest'
    
    def auto_select_model(self):
        """Automatically select best model for the target code"""
        target = self.target_var.get().strip()
        if not target:
            return
        
        language = self.detect_dominant_language(target)
        if language:
            best_model = self.get_best_model_for_language(language)
            self.model_var.set(best_model)
            self.status_var.set(f"Auto-selected {best_model} for {language.title()} code")
        else:
            self.status_var.set("Could not detect code language for auto-selection")
    
    def on_target_change(self, *args):
        """Called when target path changes"""
        # Auto-suggest after a short delay to avoid constant updates while typing
        if hasattr(self, '_target_timer'):
            self.root.after_cancel(self._target_timer)
        self._target_timer = self.root.after(1000, self.auto_select_model)
    
    def load_previous_results(self):
        """Load previous analysis results from file"""
        file_path = filedialog.askopenfilename(
            title="Load Previous Analysis Results",
            filetypes=[("Markdown files", "*.md"), ("Text files", "*.txt"), ("All files", "*.*")]
        )
        if file_path:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                self.clear_output()
                self.append_output(f"📂 Loaded results from: {os.path.basename(file_path)}\n")
                self.append_output("=" * 50 + "\n")
                self.append_output(content)
                
                # Try to extract analyzed files from the report
                self.extract_analyzed_files_from_report(content, os.path.dirname(file_path))
                
                if self.analyzed_files:
                    self.fix_button.config(state=tk.NORMAL)
                    self.status_var.set(f"Loaded results - {len(self.analyzed_files)} files ready for fixing")
                else:
                    self.status_var.set("Results loaded - No fixable files detected")
                
            except Exception as e:
                messagebox.showerror("Error", f"Could not load file: {e}")
    
    def extract_analyzed_files_from_report(self, content, base_dir):
        """Extract file paths from analysis report"""
        import re
        self.analyzed_files = []
        
        # Look for file patterns in the report
        file_patterns = [
            r'Results for ([^:\n]+):',
            r'Analyzing: ([^\n]+)',
            r'Fixed: ([^\n]+)',
            r'File: ([^\n]+)'
        ]
        
        for pattern in file_patterns:
            matches = re.findall(pattern, content)
            for match in matches:
                # Clean up the filename
                filename = match.strip()
                if filename and not filename.startswith('/'):
                    full_path = os.path.join(base_dir, filename)
                    if os.path.exists(full_path) and full_path not in self.analyzed_files:
                        self.analyzed_files.append(full_path)
    
    def start_fix_only(self):
        """Start fix-only mode for previously analyzed files"""
        if not self.analyzed_files:
            messagebox.showwarning("No Files", "No analyzed files available for fixing.\nRun analysis first or load previous results.")
            return
        
        if self.analysis_running:
            return
        
        # Confirm fix action
        result = messagebox.askyesno(
            "Fix Issues Confirmation", 
            f"This will apply fixes to {len(self.analyzed_files)} previously analyzed files.\n\n"
            "⚠️ IMPORTANT: This will modify your files!\n\n"
            "Make sure you have backups or are using version control.\n\n"
            "Do you want to continue?"
        )
        
        if not result:
            return
        
        # Clean model name
        model = self.model_var.get().replace('🚀 ', '').strip()
        
        self.analysis_running = True
        self.analyze_button.config(state=tk.DISABLED)
        self.fix_button.config(state=tk.DISABLED)
        self.autofix_button.config(state=tk.DISABLED)
        self.stop_button.config(state=tk.NORMAL)
        self.progress.start()
        self.status_var.set("Applying fixes to analyzed files...")
        
        # Clear previous output
        self.clear_output()
        
        # Run fix in separate thread
        thread = threading.Thread(target=self.run_fix_only)
        thread.daemon = True
        thread.start()
    
    def run_fix_only(self):
        """Run fixes on previously analyzed files"""
        try:
            model = self.model_var.get().replace('🚀 ', '').strip()
            
            self.append_output("🔧 Starting Fix-Only Mode\n")
            self.append_output("=" * 50 + "\n")
            self.append_output(f"🤖 Model: {model}\n")
            self.append_output(f"📁 Files to fix: {len(self.analyzed_files)}\n")
            self.append_output("⚠️  Fix mode: Files will be modified!\n")
            self.append_output("=" * 50 + "\n\n")
            
            # Set environment
            env = os.environ.copy()
            env['OLLAMA_MODELS'] = self.models_path
            env['CUDA_VISIBLE_DEVICES'] = '0'
            
            fixed_files = 0
            
            for i, file_path in enumerate(self.analyzed_files, 1):
                if not self.analysis_running:
                    break
                
                if not os.path.exists(file_path):
                    self.append_output(f"⚠️ Skipping missing file: {os.path.basename(file_path)}\n")
                    continue
                    
                self.append_output(f"[{i}/{len(self.analyzed_files)}] 🔍 Fixing: {os.path.basename(file_path)}\n")
                
                try:
                    # Read original file content
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                        original_content = f.read()
                    
                    if len(original_content) > 10000:  # Skip very large files
                        self.append_output(f"⚠️ Skipping large file: {os.path.basename(file_path)}\n\n")
                        continue
                    
                    # Create fix prompt
                    prompt = self.create_fix_prompt(file_path, original_content, 'cleanup')
                    
                    # Get fixed code from Ollama
                    self.current_process = subprocess.Popen(
                        ['ollama', 'run', model],
                        stdin=subprocess.PIPE,
                        stdout=subprocess.PIPE,
                        stderr=subprocess.STDOUT,
                        text=True,
                        env=env
                    )
                    
                    stdout, _ = self.current_process.communicate(input=prompt, timeout=300)
                    
                    if stdout:
                        # Clean up terminal control sequences first
                        clean_stdout = self.clean_terminal_output(stdout)
                        # Extract fixed code from AI response
                        fixed_code = self.extract_code_from_response(clean_stdout, original_content)
                        
                        if fixed_code and fixed_code != original_content:
                            # Create backup
                            backup_path = file_path + '.backup'
                            with open(backup_path, 'w', encoding='utf-8') as f:
                                f.write(original_content)
                            
                            # Write fixed code
                            with open(file_path, 'w', encoding='utf-8') as f:
                                f.write(fixed_code)
                            
                            self.append_output(f"✅ Fixed: {os.path.basename(file_path)} (backup created)\n")
                            fixed_files += 1
                        else:
                            self.append_output(f"ℹ️ No fixes needed: {os.path.basename(file_path)}\n")
                    
                except subprocess.TimeoutExpired:
                    self.append_output(f"⏱️ Timeout fixing: {os.path.basename(file_path)}\n")
                    if self.current_process:
                        self.current_process.kill()
                except Exception as e:
                    self.append_output(f"❌ Error fixing {os.path.basename(file_path)}: {str(e)}\n")
                
                self.append_output("\n")
            
            self.append_output("=" * 50 + "\n")
            self.append_output(f"🎉 Fix-only complete! Fixed {fixed_files}/{len(self.analyzed_files)} files\n")
            self.append_output("💾 Backup files created with .backup extension\n")
            
        except Exception as e:
            self.append_output(f"\n❌ Fix-only error: {e}\n")
        finally:
            self.root.after(0, self.analysis_finished)
    
    def extract_code_from_response(self, response, original_content):
        """Extract code from AI response"""
        import re
        
        # Look for code blocks
        code_blocks = re.findall(r'```(?:\w+)?\n(.*?)```', response, re.DOTALL)
        
        if code_blocks:
            # Use the largest code block
            fixed_code = max(code_blocks, key=len).strip()
            
            # Basic validation - should be similar length and have some common elements
            if len(fixed_code) > len(original_content) * 0.5 and len(fixed_code) < len(original_content) * 2:
                return fixed_code
        
        return None
    
    def analysis_finished(self):
        """Called when analysis is complete"""
        self.analysis_running = False
        self.analyze_button.config(state=tk.NORMAL)
        self.autofix_button.config(state=tk.NORMAL)
        
        # Enable Fix button only if we have analyzed files
        if self.analyzed_files:
            self.fix_button.config(state=tk.NORMAL)
            self.status_var.set(f"Analysis complete - {len(self.analyzed_files)} files analyzed. Click 'Fix Issues' to apply fixes.")
        else:
            self.fix_button.config(state=tk.DISABLED)
            self.status_var.set("Analysis complete - Ready for new analysis")
        
        self.stop_button.config(state=tk.DISABLED)
        self.progress.stop()
        self.current_process = None

def main():
    root = tk.Tk()
    app = OllamaCodeCheckerGUI(root)
    root.mainloop()

if __name__ == "__main__":
    main()