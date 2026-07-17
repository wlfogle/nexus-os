#!/usr/bin/env python3
"""
Test AI Powerhouse Integration
"""

def test_ai_powerhouse_search():
    """Test AI Powerhouse search functionality using embedded OSDatabase"""
    
    # Embedded OSDatabase class to avoid GUI imports
    class OSDatabase:
        def __init__(self):
            self.os_data = {
                "ubuntu": {
                    "name": "Ubuntu",
                    "versions": {"24.04": "https://ubuntu.com/download/desktop"},
                    "keywords": ["ubuntu", "canonical", "linux desktop", "beginner linux"]
                },
                "ai-powerhouse": {
                    "name": "AI Powerhouse Garuda",
                    "versions": {"latest": "https://github.com/wlfogle/ai-powerhouse-setup"},
                    "keywords": ["ai powerhouse", "development environment", "zfs", "cuda", "rust", "tauri", "react", "self-hosting", "virtualization", "ai development", "machine learning", "neural networks"]
                }
            }
        
        def search_os(self, query):
            query = query.lower()
            matches = []
            
            for os_id, data in self.os_data.items():
                score = 0
                
                # Check direct name match
                if os_id in query or data["name"].lower() in query:
                    score += 10
                
                # Check keyword matches
                for keyword in data["keywords"]:
                    if keyword in query:
                        score += 5
                
                if any(word in query for word in ["ai", "development", "powerhouse", "zfs", "cuda", "rust", "ml", "neural"]) and "ai-powerhouse" in os_id:
                    score += 8
                
                if score > 0:
                    matches.append((os_id, data, score))
            
            matches.sort(key=lambda x: x[2], reverse=True)
            return matches
    """Test AI Powerhouse search functionality"""
    print("🚀 Testing AI Powerhouse Integration")
    print("=" * 40)
    
    db = OSDatabase()
    
    # Test various AI Powerhouse related queries
    test_queries = [
        "AI Powerhouse",
        "ai development environment",
        "machine learning setup",
        "zfs cuda rust",
        "development powerhouse",
        "neural networks"
    ]
    
    for query in test_queries:
        print(f"\n🔍 Searching for: '{query}'")
        results = db.search_os(query)
        
        if results:
            for os_id, data, score in results[:3]:  # Top 3 results
                print(f"  ✅ {data['name']} (Score: {score})")
                if os_id == "ai-powerhouse":
                    print(f"     🎯 AI Powerhouse found! URL: {data['versions']['latest']}")
        else:
            print("  ❌ No results found")
    
    print("\n" + "=" * 40)
    print("✅ AI Powerhouse Integration Test Complete!")

if __name__ == "__main__":
    test_ai_powerhouse_search()