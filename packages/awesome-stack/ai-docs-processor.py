#!/usr/bin/env python3
"""
AI-Powered Document Processor using CT-900
Reads, understands, and extracts information from documents, scripts, and code
Creates comprehensive consolidated documents with all unique information
"""

import os
import json
import asyncio
import aiohttp
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Set, Any
import hashlib
import re

class AIDocumentProcessor:
    def __init__(self, docs_path: str):
        self.docs_path = Path(docs_path)
        self.output_path = self.docs_path / "_ai_processed"
        self.archive_path = self.docs_path / "_ai_archive"
        self.ollama_host = "192.168.122.172:11434"
        self.model = "codellama:7b"
        
        # Create output directories
        self.output_path.mkdir(exist_ok=True)
        self.archive_path.mkdir(exist_ok=True)
        
        # Storage for analysis results
        self.documents = []
        self.extracted_info = {}
        self.consolidated_docs = {}
        
    async def analyze_all_files(self):
        """Analyze all files using AI to understand content"""
        print(f"🔍 AI Analysis starting on: {self.docs_path}")
        
        # Find all relevant files
        file_patterns = ['*.md', '*.txt', '*.sh', '*.py', '*.js', '*.rs', '*.yml', '*.yaml', '*.json', '*.toml']
        all_files = []
        
        for pattern in file_patterns:
            all_files.extend(self.docs_path.glob(f"**/{pattern}"))
        
        # Filter out our output directories
        files_to_process = [
            f for f in all_files 
            if not str(f).startswith(str(self.output_path)) 
            and not str(f).startswith(str(self.archive_path))
        ]
        
        print(f"📄 Found {len(files_to_process)} files to analyze")
        
        # Process each file with AI
        for file_path in files_to_process:
            try:
                await self.analyze_file_with_ai(file_path)
            except Exception as e:
                print(f"❌ Error processing {file_path.name}: {str(e)}")
        
        print(f"✅ AI analysis complete on {len(self.documents)} files")
        return self.documents
    
    async def analyze_file_with_ai(self, file_path: Path):
        """Use AI to analyze and understand a single file"""
        try:
            # Read file content
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
        except UnicodeDecodeError:
            try:
                with open(file_path, 'r', encoding='latin-1') as f:
                    content = f.read()
            except:
                print(f"⚠️ Cannot read {file_path.name}, skipping")
                return
        
        # Determine file type and create appropriate analysis prompt
        file_type = self.determine_file_type(file_path)
        analysis_prompt = self.create_analysis_prompt(file_path.name, content, file_type)
        
        # Send to AI for analysis
        try:
            ai_analysis = await self.query_ai(analysis_prompt)
            
            # Parse AI response
            extracted_data = self.parse_ai_analysis(ai_analysis, file_path, content, file_type)
            
            self.documents.append(extracted_data)
            print(f"📊 Analyzed: {file_path.name} ({file_type})")
            
        except Exception as e:
            print(f"❌ AI analysis failed for {file_path.name}: {str(e)}")
    
    def determine_file_type(self, file_path: Path) -> str:
        """Determine the type of file for appropriate AI analysis"""
        suffix = file_path.suffix.lower()
        name = file_path.name.lower()
        
        type_mapping = {
            '.md': 'markdown_documentation',
            '.txt': 'text_documentation',
            '.sh': 'shell_script',
            '.py': 'python_code',
            '.js': 'javascript_code',
            '.rs': 'rust_code',
            '.yml': 'yaml_configuration',
            '.yaml': 'yaml_configuration',
            '.json': 'json_configuration',
            '.toml': 'toml_configuration'
        }
        
        return type_mapping.get(suffix, 'unknown_file')
    
    def create_analysis_prompt(self, filename: str, content: str, file_type: str) -> str:
        """Create appropriate AI analysis prompt based on file type"""
        
        base_prompt = f"""
Analyze this {file_type} file named "{filename}". 

File content:
```
{content}
```

Please provide a comprehensive analysis in JSON format with these fields:
"""
        
        if file_type == 'markdown_documentation':
            prompt = base_prompt + """
{
  "file_type": "documentation",
  "title": "Main title/purpose of this document",
  "category": "project_management/ai_systems/smart_home/infrastructure/media_stack/optimization/configuration/general",
  "key_topics": ["list", "of", "main", "topics"],
  "important_information": {
    "setup_instructions": ["step by step instructions if any"],
    "configuration_details": ["configuration information"],
    "requirements": ["dependencies or requirements"],
    "troubleshooting": ["common issues and solutions"],
    "urls_and_endpoints": ["any URLs, IPs, or endpoints mentioned"],
    "version_info": ["version numbers or dates mentioned"],
    "status_indicators": ["completion status, todos, warnings"]
  },
  "technical_details": {
    "technologies": ["docker", "proxmox", "etc"],
    "ports": ["8080", "3000", "etc"],
    "paths": ["/path/to/files"],
    "commands": ["important commands mentioned"]
  },
  "relationships": ["files or systems this relates to"],
  "outdated_indicators": ["any signs this might be outdated"],
  "duplicate_content_hash": "brief summary for duplicate detection"
}
"""
        elif file_type in ['shell_script', 'python_code', 'javascript_code', 'rust_code']:
            prompt = base_prompt + """
{
  "file_type": "code",
  "language": "bash/python/javascript/rust",
  "purpose": "What this code/script does",
  "category": "deployment/optimization/automation/utility/configuration",
  "functionality": {
    "main_functions": ["list of main functions or operations"],
    "inputs": ["what inputs it expects"],
    "outputs": ["what it produces or changes"],
    "side_effects": ["files created, services started, etc"]
  },
  "technical_details": {
    "dependencies": ["required packages, services, or tools"],
    "configuration": ["config files or environment variables used"],
    "ports": ["network ports used"],
    "paths": ["important file paths"],
    "apis": ["external APIs or services called"],
    "docker_services": ["docker containers or services involved"]
  },
  "usage_instructions": {
    "how_to_run": "command or steps to execute",
    "parameters": ["command line arguments or configuration needed"],
    "prerequisites": ["what needs to be set up first"]
  },
  "integration": ["how this connects to other parts of the system"],
  "security_considerations": ["any security implications"],
  "maintenance": ["updates or maintenance requirements"]
}
"""
        elif file_type in ['yaml_configuration', 'json_configuration', 'toml_configuration']:
            prompt = base_prompt + """
{
  "file_type": "configuration",
  "config_type": "docker-compose/application-config/service-config",
  "purpose": "What this configuration is for",
  "services_defined": ["list of services or components"],
  "technical_details": {
    "ports": ["exposed ports"],
    "volumes": ["volume mappings"],
    "networks": ["network configurations"],
    "environment_variables": ["important env vars"],
    "dependencies": ["service dependencies"],
    "images": ["docker images used"]
  },
  "deployment_info": {
    "how_to_deploy": "deployment instructions",
    "requirements": ["system requirements"],
    "configuration_files_needed": ["additional config files"]
  },
  "integration": ["how this fits into the larger system"],
  "customization": ["what can be customized"]
}
"""
        else:
            prompt = base_prompt + """
{
  "file_type": "general",
  "purpose": "What this file is for",
  "category": "general",
  "key_information": ["important points from the file"],
  "technical_details": {},
  "relationships": []
}
"""
        
        return prompt
    
    async def query_ai(self, prompt: str) -> str:
        """Send query to AI model on CT-900"""
        url = f"http://{self.ollama_host}/api/generate"
        
        payload = {
            "model": self.model,
            "prompt": prompt,
            "stream": False,
            "options": {
                "temperature": 0.1,
                "top_p": 0.9,
                "max_tokens": 2048
            }
        }
        
        async with aiohttp.ClientSession() as session:
            async with session.post(url, json=payload) as response:
                if response.status == 200:
                    result = await response.json()
                    return result.get('response', '')
                else:
                    raise Exception(f"AI request failed with status {response.status}")
    
    def parse_ai_analysis(self, ai_response: str, file_path: Path, content: str, file_type: str) -> Dict:
        """Parse AI analysis response and structure the data"""
        try:
            # Try to extract JSON from AI response
            json_match = re.search(r'\{.*\}', ai_response, re.DOTALL)
            if json_match:
                ai_data = json.loads(json_match.group())
            else:
                # Fallback if JSON parsing fails
                ai_data = {"error": "Could not parse AI response", "raw_response": ai_response}
        except json.JSONDecodeError:
            ai_data = {"error": "Invalid JSON in AI response", "raw_response": ai_response}
        
        # Combine with file metadata
        return {
            'path': file_path,
            'filename': file_path.name,
            'file_type': file_type,
            'size': file_path.stat().st_size,
            'modified': datetime.fromtimestamp(file_path.stat().st_mtime),
            'content': content,
            'content_hash': hashlib.md5(content.encode()).hexdigest(),
            'ai_analysis': ai_data
        }
    
    async def extract_unique_information(self):
        """Extract unique information across all documents using AI"""
        print("🧠 Extracting unique information across all documents...")
        
        # Group documents by category
        categories = {}
        for doc in self.documents:
            ai_analysis = doc.get('ai_analysis', {})
            category = ai_analysis.get('category', 'general')
            
            if category not in categories:
                categories[category] = []
            categories[category].append(doc)
        
        # Process each category
        for category, docs in categories.items():
            if len(docs) > 1:
                print(f"📋 Processing {category} category with {len(docs)} documents")
                consolidated = await self.consolidate_category_information(category, docs)
                self.consolidated_docs[category] = consolidated
            else:
                # Single document categories
                self.consolidated_docs[category] = docs[0] if docs else None
        
        return self.consolidated_docs
    
    async def consolidate_category_information(self, category: str, documents: List[Dict]) -> Dict:
        """Use AI to consolidate information from multiple documents in a category"""
        
        # Create consolidation prompt
        doc_summaries = []
        for doc in documents:
            ai_analysis = doc.get('ai_analysis', {})
            summary = {
                'filename': doc['filename'],
                'file_type': doc['file_type'],
                'analysis': ai_analysis
            }
            doc_summaries.append(summary)
        
        consolidation_prompt = f"""
I have {len(documents)} documents in the "{category}" category that need to be consolidated into one comprehensive document.

Document summaries:
{json.dumps(doc_summaries, indent=2, default=str)}

Please create a comprehensive consolidation plan in JSON format:

{{
  "consolidated_title": "Title for the consolidated document",
  "document_structure": {{
    "sections": [
      {{
        "section_title": "Section name",
        "content_sources": ["filename1.md", "filename2.md"],
        "unique_information": "What unique info to extract from each source",
        "avoid_duplication": "What duplicate information to exclude"
      }}
    ]
  }},
  "information_extraction": {{
    "setup_instructions": "Consolidated setup steps removing duplicates",
    "configuration": "Merged configuration information",
    "troubleshooting": "Combined troubleshooting information",
    "technical_specifications": "All technical details consolidated",
    "requirements": "Complete requirements list",
    "integration_points": "How everything connects together"
  }},
  "files_to_archive": ["list of files that are now redundant"],
  "cross_references": "How this relates to other categories"
}}

Focus on:
1. Removing duplicate information
2. Combining related information
3. Creating a logical flow
4. Preserving all unique details
5. Identifying outdated information
"""
        
        try:
            consolidation_plan = await self.query_ai(consolidation_prompt)
            
            # Parse the consolidation plan
            json_match = re.search(r'\{.*\}', consolidation_plan, re.DOTALL)
            if json_match:
                plan = json.loads(json_match.group())
                
                # Now create the actual consolidated document
                consolidated_content = await self.create_consolidated_document(category, documents, plan)
                
                return {
                    'category': category,
                    'plan': plan,
                    'consolidated_content': consolidated_content,
                    'source_documents': [doc['filename'] for doc in documents],
                    'created': datetime.now().isoformat()
                }
            else:
                raise Exception("Could not parse consolidation plan")
                
        except Exception as e:
            print(f"❌ Consolidation failed for {category}: {str(e)}")
            return {
                'category': category,
                'error': str(e),
                'source_documents': [doc['filename'] for doc in documents],
                'created': datetime.now().isoformat()
            }
    
    async def create_consolidated_document(self, category: str, documents: List[Dict], plan: Dict) -> str:
        """Create the actual consolidated document content using AI"""
        
        # Prepare full document contents for AI
        full_contents = []
        for doc in documents:
            full_contents.append(f"=== {doc['filename']} ===\n{doc['content']}\n")
        
        creation_prompt = f"""
Create a comprehensive consolidated document for the "{category}" category.

Consolidation Plan:
{json.dumps(plan, indent=2, default=str)}

Source Documents:
{chr(10).join(full_contents)}

Please create a well-structured markdown document that:
1. Follows the consolidation plan structure
2. Includes ALL unique information from source documents
3. Removes duplicate information
4. Creates logical sections and flow
5. Includes proper headers, lists, and formatting
6. Preserves important technical details, commands, configurations
7. Includes troubleshooting and setup instructions
8. Cross-references related information

Output the complete markdown document:
"""
        
        try:
            consolidated_doc = await self.query_ai(creation_prompt)
            return consolidated_doc
        except Exception as e:
            print(f"❌ Document creation failed for {category}: {str(e)}")
            return f"# {category.replace('_', ' ').title()}\n\nError creating consolidated document: {str(e)}"
    
    async def save_consolidated_documents(self):
        """Save all consolidated documents to files"""
        print("💾 Saving consolidated documents...")
        
        consolidated_dir = self.output_path / "consolidated"
        consolidated_dir.mkdir(exist_ok=True)
        
        summary_info = []
        
        for category, consolidated_data in self.consolidated_docs.items():
            if consolidated_data and 'consolidated_content' in consolidated_data:
                filename = f"{category}_comprehensive.md"
                file_path = consolidated_dir / filename
                
                with open(file_path, 'w', encoding='utf-8') as f:
                    f.write(consolidated_data['consolidated_content'])
                
                print(f"📄 Created: {filename}")
                
                summary_info.append({
                    'category': category,
                    'filename': filename,
                    'source_count': len(consolidated_data.get('source_documents', [])),
                    'source_files': consolidated_data.get('source_documents', [])
                })
        
        # Create summary report
        await self.create_summary_report(summary_info)
        
        return summary_info
    
    async def create_summary_report(self, summary_info: List[Dict]):
        """Create a comprehensive summary report"""
        report_content = [
            "# AI Document Processing Report",
            f"**Generated**: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
            f"**AI Model**: {self.model} on {self.ollama_host}",
            f"**Source Path**: {self.docs_path}",
            "",
            "## Processing Summary",
            f"- **Total Files Processed**: {len(self.documents)}",
            f"- **Categories Identified**: {len(self.consolidated_docs)}",
            f"- **Consolidated Documents**: {len(summary_info)}",
            "",
            "## Consolidated Documents Created"
        ]
        
        for info in summary_info:
            report_content.extend([
                f"### {info['category'].replace('_', ' ').title()}",
                f"- **Output File**: `{info['filename']}`",
                f"- **Source Documents**: {info['source_count']} files",
                f"- **Sources**: {', '.join(info['source_files'])}",
                ""
            ])
        
        report_content.extend([
            "## File Type Analysis",
            "| File Type | Count |",
            "|-----------|-------|"
        ])
        
        # Count file types
        type_counts = {}
        for doc in self.documents:
            file_type = doc['file_type']
            type_counts[file_type] = type_counts.get(file_type, 0) + 1
        
        for file_type, count in type_counts.items():
            report_content.append(f"| {file_type.replace('_', ' ').title()} | {count} |")
        
        report_content.extend([
            "",
            "## AI Analysis Quality",
            f"- **Successful Analyses**: {len([d for d in self.documents if 'error' not in d.get('ai_analysis', {})])}",
            f"- **Failed Analyses**: {len([d for d in self.documents if 'error' in d.get('ai_analysis', {})])}",
            "",
            "---",
            "",
            "All original files have been preserved. Check the consolidated documents for comprehensive information."
        ])
        
        report_path = self.output_path / "ai_processing_report.md"
        with open(report_path, 'w', encoding='utf-8') as f:
            f.write('\n'.join(report_content))
        
        print(f"📊 Summary report saved: {report_path}")

async def main():
    """Main function to run the AI document processor"""
    docs_path = "/home/lou/awesome_stack/docs"
    
    if not os.path.exists(docs_path):
        print(f"❌ Directory not found: {docs_path}")
        return
    
    processor = AIDocumentProcessor(docs_path)
    
    print("🚀 Starting AI-powered document processing...")
    print(f"🧠 Using AI model: {processor.model} on {processor.ollama_host}")
    
    try:
        # Test AI connection
        test_response = await processor.query_ai("Hello, please respond with 'AI connection successful'")
        if "successful" not in test_response.lower():
            print("⚠️ AI connection may have issues, but continuing...")
        else:
            print("✅ AI connection verified")
    except Exception as e:
        print(f"❌ Cannot connect to AI: {str(e)}")
        print("Please ensure CT-900 is running and Ollama is accessible")
        return
    
    # Step 1: Analyze all files with AI
    print("\n📊 Step 1: AI analysis of all files...")
    await processor.analyze_all_files()
    
    # Step 2: Extract unique information
    print("\n🧠 Step 2: Extracting unique information...")
    await processor.extract_unique_information()
    
    # Step 3: Save consolidated documents
    print("\n💾 Step 3: Creating consolidated documents...")
    summary = await processor.save_consolidated_documents()
    
    print(f"\n✅ AI processing complete!")
    print(f"📂 Consolidated documents: {processor.output_path}/consolidated/")
    print(f"📊 Processing report: {processor.output_path}/ai_processing_report.md")
    print(f"🧠 Created {len(summary)} comprehensive documents")

if __name__ == "__main__":
    asyncio.run(main())
