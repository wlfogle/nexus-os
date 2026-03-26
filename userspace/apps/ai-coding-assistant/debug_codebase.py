#!/usr/bin/env python3

import requests
import json
import sys

def analyze_code_with_ai(code_snippet, language="rust", focus="bugs and security issues"):
    """Send code to AI on CT-900 for analysis"""
    
    prompt = f"""As an expert {language} developer, analyze this code for {focus}:

```{language}
{code_snippet}
```

Please identify:
1. Potential bugs or logic errors
2. Security vulnerabilities 
3. Performance issues
4. Memory safety concerns
5. Best practice violations

Provide specific, actionable recommendations with code examples where possible."""

    payload = {
        "model": "magicoder:7b",  # Using MagiCoder for better code analysis
        "prompt": prompt,
        "stream": False,
        "options": {
            "temperature": 0.1,
            "top_p": 0.9
        }
    }
    
    try:
        response = requests.post("http://192.168.122.172:11434/api/generate", 
                               json=payload, 
                               timeout=60)
        response.raise_for_status()
        
        result = response.json()
        return result.get('response', 'No response received')
        
    except requests.exceptions.RequestException as e:
        return f"Error connecting to AI: {e}"

def main():
    # Read the lib.rs file for analysis
    try:
        with open('src-tauri/src/lib.rs', 'r') as f:
            lib_code = f.read()
    except FileNotFoundError:
        print("Could not find src-tauri/src/lib.rs")
        return

    print("🔍 Analyzing lib.rs with AI on CT-900...")
    print("=" * 80)
    
    analysis = analyze_code_with_ai(lib_code, "rust", "bugs, security vulnerabilities, and improvements")
    print(analysis)
    
    print("\n" + "=" * 80)
    print("🔍 Analyzing optimized_lib.rs with AI...")
    print("=" * 80)
    
    # Read optimized_lib.rs
    try:
        with open('src-tauri/src/optimized_lib.rs', 'r') as f:
            optimized_code = f.read()
            
        # Analyze first 2000 characters to stay within token limits
        optimized_snippet = optimized_code[:2000] + "..." if len(optimized_code) > 2000 else optimized_code
        
        analysis2 = analyze_code_with_ai(optimized_snippet, "rust", "async/await patterns, performance issues, and memory management")
        print(analysis2)
        
    except FileNotFoundError:
        print("Could not find src-tauri/src/optimized_lib.rs")

if __name__ == "__main__":
    main()
