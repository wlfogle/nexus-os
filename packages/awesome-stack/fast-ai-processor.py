#!/usr/bin/env python3
"""
Fast AI-Powered Document Processor
Efficiently processes documents in batches using CT-900
"""

import os
import json
import asyncio
import aiohttp
from pathlib import Path
from datetime import datetime
from typing import List, Dict
import hashlib
import re

class FastAIProcessor:
    def __init__(self, docs_path: str):
        self.docs_path = Path(docs_path)
        self.output_path = self.docs_path / "_ai_processed"
        self.ollama_host = "192.168.122.172:11434"
        self.model = "codellama:7b"
        self.output_path.mkdir(exist_ok=True)
        self.processed_files = []
        
    async def process_documents_efficiently(self):
        """Process documents efficiently with batch analysis"""
        print(f"🔍 Fast AI processing on: {self.docs_path}")
        
        # Group files by type for batch processing
        file_groups = self.group_files_by_type()
        
        # Process each group
        for file_type, files in file_groups.items():
            if files:
                print(f"📊 Processing {len(files)} {file_type} files...")
                await self.process_file_group(file_type, files)
        
        # Create consolidated documents
        await self.create_consolidated_documents()
        
        return self.processed_files
    
    def group_files_by_type(self) -> Dict[str, List[Path]]:
        """Group similar files together for batch processing"""
        groups = {
            'alexa_docs': [],
            'ai_docs': [],
            'infrastructure_docs': [],
            'media_docs': [],
            'optimization_docs': [],
            'project_docs': [],
            'scripts': [],
            'configs': []
        }
        
        # Find all relevant files
        all_files = []
        for pattern in ['*.md', '*.txt', '*.sh', '*.yml', '*.yaml', '*.json']:
            all_files.extend(self.docs_path.glob(f"**/{pattern}"))
        
        # Filter and categorize
        for file_path in all_files:
            if str(file_path).startswith(str(self.output_path)):
                continue
                
            filename = file_path.name.lower()
            
            # Categorize based on filename
            if 'alexa' in filename or 'smart' in filename or 'voice' in filename:
                groups['alexa_docs'].append(file_path)
            elif any(word in filename for word in ['ai', 'assistant', 'tauri', 'ollama']):
                groups['ai_docs'].append(file_path)
            elif any(word in filename for word in ['proxmox', 'vm', 'container', 'android']):
                groups['infrastructure_docs'].append(file_path)
            elif any(word in filename for word in ['media', 'grandma', 'plex', 'jellyfin', 'radarr', 'sonarr']):
                groups['media_docs'].append(file_path)
            elif any(word in filename for word in ['optimization', 'performance', 'hardware', 'ultimate']):
                groups['optimization_docs'].append(file_path)
            elif any(word in filename for word in ['project', 'plan', 'summary', 'implementation']):
                groups['project_docs'].append(file_path)
            elif file_path.suffix in ['.sh', '.py', '.js']:
                groups['scripts'].append(file_path)
            elif file_path.suffix in ['.yml', '.yaml', '.json']:
                groups['configs'].append(file_path)
        
        return groups
    
    async def process_file_group(self, group_type: str, files: List[Path]):
        """Process a group of similar files together"""
        # Read all files in the group
        file_contents = {}
        for file_path in files:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                file_contents[file_path.name] = {
                    'path': file_path,
                    'content': content,
                    'size': len(content)
                }
            except Exception as e:
                print(f"⚠️ Cannot read {file_path.name}: {str(e)}")
        
        if not file_contents:
            return
        
        # Create batch analysis prompt
        batch_prompt = self.create_batch_analysis_prompt(group_type, file_contents)
        
        try:
            # Get AI analysis for the entire group
            ai_response = await self.query_ai(batch_prompt)
            
            # Parse and save results
            analysis_result = {
                'group_type': group_type,
                'files_analyzed': list(file_contents.keys()),
                'ai_analysis': ai_response,
                'processed_at': datetime.now().isoformat()
            }
            
            # Save individual file info
            for filename, file_info in file_contents.items():
                self.processed_files.append({
                    'filename': filename,
                    'path': file_info['path'],
                    'group_type': group_type,
                    'content': file_info['content'],
                    'size': file_info['size']
                })
            
            # Save group analysis
            analysis_file = self.output_path / f"{group_type}_analysis.json"
            with open(analysis_file, 'w') as f:
                json.dump(analysis_result, f, indent=2, default=str)
            
            print(f"✅ Analyzed {group_type}: {len(file_contents)} files")
            
        except Exception as e:
            print(f"❌ Failed to analyze {group_type}: {str(e)}")
    
    def create_batch_analysis_prompt(self, group_type: str, file_contents: Dict) -> str:
        """Create efficient batch analysis prompt"""
        files_summary = []
        for filename, info in file_contents.items():
            # Use first 500 chars as preview
            preview = info['content'][:500] + "..." if len(info['content']) > 500 else info['content']
            files_summary.append(f"FILE: {filename}\nCONTENT PREVIEW:\n{preview}\n")
        
        return f"""
Analyze these {group_type} files as a group. Provide a comprehensive analysis focusing on:

1. MAIN PURPOSE: What is this group of files for?
2. KEY INFORMATION: Most important information across all files
3. SETUP INSTRUCTIONS: Step-by-step setup from all files combined
4. CONFIGURATION: All configuration details
5. REQUIREMENTS: Dependencies and prerequisites  
6. TROUBLESHOOTING: Common issues and solutions
7. TECHNICAL DETAILS: Ports, paths, commands, URLs
8. INTEGRATION: How these connect to other parts
9. DUPLICATES: Which files have duplicate information
10. OUTDATED: Any outdated information to remove

FILES TO ANALYZE:
{chr(10).join(files_summary)}

Provide analysis in this format:

## {group_type.replace('_', ' ').title()} Analysis

### Main Purpose
[What this group is for]

### Key Information
- [Important point 1]
- [Important point 2]
- [etc...]

### Complete Setup Instructions
1. [Step 1]
2. [Step 2]
[etc...]

### Configuration Details
- [Config 1]: [Details]
- [Config 2]: [Details]

### Requirements
- [Requirement 1]
- [Requirement 2]

### Technical Specifications
- Ports: [list]
- Paths: [list]  
- Commands: [list]
- URLs/IPs: [list]

### Troubleshooting
- Issue: [Solution]
- Issue: [Solution]

### Integration Points
[How this connects to other systems]

### Duplicate Information Found
- [File A] and [File B] both contain: [duplicate info]

### Outdated Information
- [File]: [outdated info to remove]

### Recommended Consolidation
- Merge [files] into: [suggested filename]
- Archive: [files that are redundant]
"""
    
    async def query_ai(self, prompt: str) -> str:
        """Query AI with timeout and retry logic"""
        url = f"http://{self.ollama_host}/api/generate"
        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.1,
                "max_tokens": 4096
            }
        }
        
        timeout = aiohttp.ClientTimeout(total=300)  # 5 minute timeout
        
        for attempt in range(3):
            try:
                async with aiohttp.ClientSession(timeout=timeout) as session:
                    async with session.post(url, json=payload) as response:
                        if response.status == 200:
                            result = await response.json()
                            return result.get('response', '')
                        else:
                            raise Exception(f"HTTP {response.status}")
            except Exception as e:
                if attempt == 2:  # Last attempt
                    raise e
                print(f"⚠️ Attempt {attempt + 1} failed, retrying...")
                await asyncio.sleep(5)
    
    async def create_consolidated_documents(self):
        """Create final consolidated documents based on AI analysis"""
        print("📝 Creating consolidated documents...")
        
        consolidated_dir = self.output_path / "consolidated"
        consolidated_dir.mkdir(exist_ok=True)
        
        # Read all group analyses
        group_analyses = {}
        for analysis_file in self.output_path.glob("*_analysis.json"):
            with open(analysis_file) as f:
                data = json.load(f)
                group_type = data['group_type']
                group_analyses[group_type] = data['ai_analysis']
        
        # Create individual consolidated documents
        for group_type, analysis in group_analyses.items():
            consolidated_content = f"""# {group_type.replace('_', ' ').title()} - Comprehensive Guide

**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}  
**AI Analysis**: Processed by {self.model}

{analysis}

---

*This document consolidates information from multiple source files to provide a comprehensive, non-redundant guide.*
"""
            
            filename = f"{group_type}_comprehensive.md"
            file_path = consolidated_dir / filename
            
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(consolidated_content)
            
            print(f"📄 Created: {filename}")
        
        # Create master summary
        await self.create_master_summary(group_analyses)
    
    async def create_master_summary(self, group_analyses: Dict):
        """Create a master summary of all processing"""
        summary_content = [
            "# Fast AI Document Processing - Master Summary",
            f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"**AI Model**: {self.model}",
            f"**Source Path**: {self.docs_path}",
            "",
            "## Processing Results",
            f"- **Total Files Processed**: {len(self.processed_files)}",
            f"- **Document Groups**: {len(group_analyses)}",
            "",
            "## Document Groups Processed"
        ]
        
        file_counts = {}
        for file_info in self.processed_files:
            group = file_info['group_type']
            file_counts[group] = file_counts.get(group, 0) + 1
        
        for group_type, count in file_counts.items():
            summary_content.extend([
                f"### {group_type.replace('_', ' ').title()}",
                f"- **Files**: {count}",
                f"- **Output**: `{group_type}_comprehensive.md`",
                ""
            ])
        
        summary_content.extend([
            "## Key Findings",
            "",
            "Based on AI analysis, the following consolidations were recommended:",
            ""
        ])
        
        # Add AI insights about duplicates and outdated content
        for group_type, analysis in group_analyses.items():
            if "duplicate" in analysis.lower() or "outdated" in analysis.lower():
                summary_content.append(f"- **{group_type}**: Contains duplicate/outdated information")
        
        summary_content.extend([
            "",
            "## Next Steps",
            "",
            "1. Review the consolidated documents in the `consolidated/` directory",
            "2. Check AI recommendations for files that can be archived",
            "3. Use consolidated guides as primary reference documents",
            "4. Archive original files that are now redundant",
            "",
            "## Files Location",
            f"- **Consolidated Docs**: `{self.output_path}/consolidated/`",
            f"- **Group Analyses**: `{self.output_path}/*_analysis.json`",
            f"- **This Summary**: `{self.output_path}/master_summary.md`"
        ])
        
        summary_path = self.output_path / "master_summary.md"
        with open(summary_path, 'w', encoding='utf-8') as f:
            f.write('\n'.join(summary_content))
        
        print(f"📊 Master summary created: {summary_path}")

async def main():
    """Main function"""
    docs_path = "/home/lou/awesome_stack/docs"
    
    if not os.path.exists(docs_path):
        print(f"❌ Directory not found: {docs_path}")
        return
    
    processor = FastAIProcessor(docs_path)
    
    print("🚀 Starting Fast AI Document Processing...")
    print(f"🧠 Using: {processor.model} on {processor.ollama_host}")
    
    # Test connection
    try:
        test_response = await processor.query_ai("Respond with: AI Ready")
        if "ready" in test_response.lower():
            print("✅ AI connection verified")
        else:
            print("⚠️ AI connection issues, but continuing...")
    except Exception as e:
        print(f"❌ Cannot connect to AI: {str(e)}")
        return
    
    # Process documents
    await processor.process_documents_efficiently()
    
    print(f"\n✅ Fast AI processing complete!")
    print(f"📂 Results: {processor.output_path}/")
    print(f"📊 Summary: {processor.output_path}/master_summary.md")
    print(f"📄 Consolidated docs: {processor.output_path}/consolidated/")

if __name__ == "__main__":
    asyncio.run(main())
