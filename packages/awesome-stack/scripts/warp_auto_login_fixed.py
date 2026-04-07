#!/usr/bin/env python3
"""
Warp Terminal Auto-Login Script - FIXED VERSION
Fixes tab management issues where email tab closes when opening Warp tab

Key fixes:
1. Prevent email tab from closing when opening Warp
2. Better tab handle tracking and recovery
3. Explicit tab preservation strategies
4. Improved error handling for tab loss scenarios
"""

import time
import subprocess
import sys
import re
import json
import random
from selenium import webdriver
import pyperclip
from webdriver_manager.firefox import GeckoDriverManager
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.webdriver.firefox.options import Options
from selenium.common.exceptions import TimeoutException, NoSuchElementException
import pyperclip

class WarpAutoLoginFixed:
    def __init__(self, headless=False):
        self.driver = None
        self.temp_email = None
        self.headless = headless
        self.email_tab_handle = None  # Store email tab handle explicitly
        self.warp_tab_handle = None   # Store warp tab handle explicitly
        
    def preserve_email_tab(self):
        """Mark current tab as the email tab to preserve it"""
        if self.driver and self.driver.window_handles:
            self.email_tab_handle = self.driver.current_window_handle
            print(f"📌 Preserving email tab: {self.email_tab_handle[:8]}...")
            
            # Add a marker to the page to identify it later
            try:
                self.driver.execute_script("""
                window.emailTabMarker = true;
                document.title = 'EMAIL_TAB_' + document.title;
                """)
            except:
                pass
        
    def find_email_tab(self):
        """Find and return to the email tab"""
        if not self.driver or not self.driver.window_handles:
            print("❌ No browser windows available")
            return False
            
        # First try the stored handle
        if self.email_tab_handle and self.email_tab_handle in self.driver.window_handles:
            try:
                self.driver.switch_to.window(self.email_tab_handle)
                # Verify it's actually the email tab
                if self.is_email_tab():
                    print(f"✅ Found email tab via stored handle: {self.email_tab_handle[:8]}...")
                    return True
            except:
                pass
        
        # Search through all tabs for email provider
        print(f"🔍 Searching through {len(self.driver.window_handles)} tabs for email provider...")
        email_providers = ['guerrillamail', 'tempmail', 'yopmail', 'maildrop', '10minutemail']
        
        for i, handle in enumerate(self.driver.window_handles):
            try:
                self.driver.switch_to.window(handle)
                current_url = self.driver.current_url.lower()
                current_title = self.driver.title.lower()
                
                # Check URL and title for email provider indicators
                is_email = any(provider in current_url for provider in email_providers)
                is_email = is_email or any(provider in current_title for provider in email_providers)
                is_email = is_email or 'email_tab_' in current_title.lower()
                
                # Check for email tab marker
                try:
                    has_marker = self.driver.execute_script("return window.emailTabMarker === true;")
                    is_email = is_email or has_marker
                except:
                    pass
                
                if is_email:
                    self.email_tab_handle = handle
                    print(f"✅ Found email tab at position {i}: {current_url[:50]}...")
                    return True
                    
            except Exception as e:
                print(f"⚠️  Error checking tab {i}: {e}")
                continue
        
        print("❌ Could not find email tab!")
        return False
    
    def is_email_tab(self):
        """Check if current tab is an email provider"""
        try:
            current_url = self.driver.current_url.lower()
            current_title = self.driver.title.lower()
            email_providers = ['guerrillamail', 'tempmail', 'yopmail', 'maildrop', '10minutemail']
            
            is_email = any(provider in current_url for provider in email_providers)
            is_email = is_email or any(provider in current_title for provider in email_providers)
            is_email = is_email or 'email_tab_' in current_title.lower()
            
            # Check for email tab marker
            try:
                has_marker = self.driver.execute_script("return window.emailTabMarker === true;")
                is_email = is_email or has_marker
            except:
                pass
                
            return is_email
        except:
            return False
    
    def open_warp_in_new_tab(self):
        """Open Warp login in a new tab while preserving the email tab"""
        if not self.driver:
            return False
            
        # Ensure we're on the email tab first
        if not self.find_email_tab():
            print("❌ Cannot find email tab to preserve")
            return False
        
        # Mark this tab as the email tab
        self.preserve_email_tab()
        
        # Get initial tab count
        initial_tab_count = len(self.driver.window_handles)
        print(f"📊 Initial tab count: {initial_tab_count}")
        
        # Open Warp in new tab using JavaScript (most reliable method)
        print("🔄 Opening Warp login in new tab...")
        try:
            # Method 1: Open directly to Warp URL in new tab
            self.driver.execute_script("window.open('https://app.warp.dev/login/remote', '_blank');")
            time.sleep(2)  # Wait for tab to open
            
            # Verify new tab opened
            new_tab_count = len(self.driver.window_handles)
            if new_tab_count > initial_tab_count:
                # Switch to the new tab
                new_handles = [h for h in self.driver.window_handles if h != self.email_tab_handle]
                if new_handles:
                    self.warp_tab_handle = new_handles[-1]  # Get the newest tab
                    self.driver.switch_to.window(self.warp_tab_handle)
                    print(f"✅ Opened Warp in new tab: {self.warp_tab_handle[:8]}...")
                    return True
            
            print("⚠️  New tab didn't open properly, trying alternative method...")
            
        except Exception as e:
            print(f"⚠️  JavaScript method failed: {e}")
        
        # Method 2: Use keyboard shortcut to open new tab
        try:
            self.find_email_tab()  # Return to email tab
            from selenium.webdriver.common.keys import Keys
            from selenium.webdriver.common.action_chains import ActionChains
            
            # Ctrl+T to open new tab
            ActionChains(self.driver).key_down(Keys.CONTROL).send_keys('t').key_up(Keys.CONTROL).perform()
            time.sleep(1)
            
            # Check if new tab opened
            if len(self.driver.window_handles) > initial_tab_count:
                new_handles = [h for h in self.driver.window_handles if h != self.email_tab_handle]
                if new_handles:
                    self.warp_tab_handle = new_handles[-1]
                    self.driver.switch_to.window(self.warp_tab_handle)
                    self.driver.get('https://app.warp.dev/login/remote')
                    print(f"✅ Opened Warp via keyboard shortcut: {self.warp_tab_handle[:8]}...")
                    return True
                    
        except Exception as e:
            print(f"⚠️  Keyboard method failed: {e}")
        
        print("❌ Failed to open Warp in new tab")
        return False
    
    def human_delay(self, min_seconds=1, max_seconds=3):
        """Add human-like random delay"""
        delay = random.uniform(min_seconds, max_seconds)
        time.sleep(delay)
        
    def type_like_human(self, element, text):
        """Type text with human-like delays"""
        element.clear()
        time.sleep(random.uniform(0.3, 0.8))
        
        for i, char in enumerate(text):
            element.send_keys(char)
            if char in '.@':
                time.sleep(random.uniform(0.15, 0.4))
            else:
                time.sleep(random.uniform(0.05, 0.2))
            
            if random.random() < 0.1:
                time.sleep(random.uniform(0.2, 0.5))
        
    def setup_driver(self):
        """Initialize Firefox webdriver with stealth options"""
        options = Options()
        if self.headless:
            options.add_argument('--headless')
        
        # Stealth options
        options.add_argument('--no-sandbox')
        options.add_argument('--disable-dev-shm-usage')
        options.add_argument('--disable-blink-features=AutomationControlled')
        options.set_preference('dom.webdriver.enabled', False)
        options.set_preference('useAutomationExtension', False)
        options.set_preference('general.useragent.override', 
                             'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36')
        
        try:
            # Try FireDragon first
            try:
                options.binary_location = '/usr/bin/firedragon'
                self.driver = webdriver.Firefox(executable_path=GeckoDriverManager().install(), options=options)
                print("✅ Using FireDragon browser")
            except:
                # Fallback to Firefox
                options.binary_location = None
                self.driver = webdriver.Firefox(executable_path=GeckoDriverManager().install(), options=options)
                print("✅ Using Firefox browser")
                
            self.driver.implicitly_wait(10)
            return True
            
        except Exception as e:
            print(f"❌ Failed to initialize browser: {e}")
            return False
    
    def get_temp_email(self):
        """Get temporary email from Guerrilla Mail (most reliable)"""
        print("🔄 Getting temporary email from Guerrilla Mail...")
        
        try:
            self.driver.get('https://www.guerrillamail.com/')
            self.human_delay(3, 5)
            
            # Find email address
            email_element = WebDriverWait(self.driver, 10).until(
                EC.presence_of_element_located((By.CSS_SELECTOR, '#email-widget span, #email-widget'))
            )
            
            self.temp_email = email_element.text.strip()
            if not self.temp_email or '@' not in self.temp_email:
                # Try alternative method
                page_text = self.driver.page_source
                email_match = re.search(r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}', page_text)
                if email_match:
                    self.temp_email = email_match.group(0)
            
            if self.temp_email and '@' in self.temp_email:
                print(f"✅ Got temporary email: {self.temp_email}")
                pyperclip.copy(self.temp_email)
                
                # Mark this as the email tab
                self.preserve_email_tab()
                return True
            else:
                print(f"❌ Invalid email: {self.temp_email}")
                return False
                
        except Exception as e:
            print(f"❌ Failed to get temporary email: {e}")
            return False
    
    def login_to_warp(self):
        """Login to Warp in the current tab"""
        print("🔄 Logging into Warp...")
        
        try:
            # If not already on Warp, navigate there
            current_url = self.driver.current_url
            if 'app.warp.dev' not in current_url:
                self.driver.get('https://app.warp.dev/login/remote')
                self.human_delay(2, 4)
            
            # Find email input
            email_selectors = [
                ".modal-container-input",
                "input[placeholder='Your email address']",
                "input[type='email']",
                "input[placeholder*='email']"
            ]
            
            email_input = None
            for selector in email_selectors:
                try:
                    email_input = WebDriverWait(self.driver, 5).until(
                        EC.element_to_be_clickable((By.CSS_SELECTOR, selector))
                    )
                    break
                except TimeoutException:
                    continue
            
            if not email_input:
                print("❌ Could not find email input field")
                return False
            
            # Enter email
            print(f"⌨️  Entering email: {self.temp_email}")
            self.type_like_human(email_input, self.temp_email)
            
            # Find and click continue button
            button_selectors = [
                ".modal-container-button-full-width",
                "button[type='submit']",
                "//button[contains(text(), 'Continue')]"
            ]
            
            for selector in button_selectors:
                try:
                    if selector.startswith('//'):
                        button = self.driver.find_element(By.XPATH, selector)
                    else:
                        button = self.driver.find_element(By.CSS_SELECTOR, selector)
                    
                    self.human_delay(0.5, 1)
                    button.click()
                    print("✅ Clicked continue button")
                    break
                except NoSuchElementException:
                    continue
            else:
                # Fallback: press Enter
                email_input.send_keys("\n")
                print("✅ Pressed Enter to submit")
            
            self.human_delay(2, 3)
            return True
            
        except Exception as e:
            print(f"❌ Error logging into Warp: {e}")
            return False
    
    def check_email_for_verification(self, max_wait_minutes=5):
        """Check email for verification link with proper tab management"""
        print(f"🔄 Checking email for verification link...")
        
        # Return to email tab
        if not self.find_email_tab():
            print("❌ Cannot find email tab")
            return None
        
        max_checks = max_wait_minutes * 4  # Check every 15 seconds
        
        for i in range(max_checks):
            try:
                print(f"🔄 Refreshing inbox (check #{i+1})...")
                
                # Ensure we're still on the email tab
                if not self.is_email_tab():
                    if not self.find_email_tab():
                        print("❌ Lost email tab!")
                        return None
                
                self.driver.refresh()
                self.human_delay(2, 3)
                
                # Look for emails
                email_selectors = [
                    "tr[onclick]",     # Guerrilla Mail rows
                    ".email-item",     # General email items
                    "tbody tr",        # Table rows
                    "#email_list tr"   # Email list rows
                ]
                
                email_found = False
                for selector in email_selectors:
                    try:
                        emails = self.driver.find_elements(By.CSS_SELECTOR, selector)
                        if emails:
                            print(f"📧 Found {len(emails)} email(s)")
                            # Click first email
                            self.driver.execute_script("arguments[0].click();", emails[0])
                            email_found = True
                            break
                    except:
                        continue
                
                if email_found:
                    self.human_delay(1, 2)
                    
                    # Look for verification link
                    page_content = self.driver.page_source
                    
                    # Try to find clickable Firebase links first
                    try:
                        firebase_links = self.driver.find_elements(By.XPATH, 
                            "//a[contains(@href, 'firebaseapp') and contains(@href, 'auth')]")
                        if firebase_links:
                            verification_link = firebase_links[0].get_attribute('href')
                            print(f"✅ Found Firebase verification link")
                            return verification_link
                    except:
                        pass
                    
                    # Fallback to regex search
                    firebase_patterns = [
                        r'https://[a-zA-Z0-9.-]+\.firebaseapp\.com/__/auth/action\?[^\s\"\'<>\)\]]+',
                        r'https://[^\s\"\'<>\)\]]+firebaseapp[^\s\"\'<>\)\]]+auth[^\s\"\'<>\)\]]+'
                    ]
                    
                    for pattern in firebase_patterns:
                        matches = re.findall(pattern, page_content)
                        if matches:
                            print(f"✅ Found verification link via regex")
                            return matches[0]
                
                if i < max_checks - 1:
                    wait_time = 15
                    print(f"⏳ Waiting {wait_time} seconds before next check... ({i+1}/{max_checks})")
                    time.sleep(wait_time)
                
            except Exception as e:
                print(f"⚠️  Error checking email: {e}")
                time.sleep(10)
        
        print("❌ Timeout waiting for verification email")
        return None
    
    def complete_firebase_auth(self, verification_link):
        """Complete Firebase auth in new tab while preserving existing tabs"""
        print("🔄 Completing Firebase authentication...")
        
        try:
            # Store current tab handles
            initial_handles = self.driver.window_handles.copy()
            print(f"📊 Initial tabs: {len(initial_handles)}")
            
            # Open verification link in new tab
            self.driver.execute_script(f"window.open('{verification_link}', '_blank');")
            time.sleep(2)
            
            # Find new tab
            new_handles = self.driver.window_handles
            firebase_tab = None
            for handle in new_handles:
                if handle not in initial_handles:
                    firebase_tab = handle
                    break
            
            if not firebase_tab:
                print("❌ Could not find Firebase auth tab")
                return False
            
            # Switch to Firebase tab
            self.driver.switch_to.window(firebase_tab)
            print("🔄 Switched to Firebase auth tab")
            
            # Wait for auth to complete
            self.human_delay(3, 5)
            
            # Look for success or redirect
            current_url = self.driver.current_url
            print(f"✅ Firebase auth completed: {current_url[:60]}...")
            
            return True
            
        except Exception as e:
            print(f"❌ Error in Firebase auth: {e}")
            return False
    
    def cleanup(self):
        """Clean up resources"""
        if self.driver:
            print("🔄 Closing browser...")
            self.driver.quit()
    
    def run(self):
        """Main execution flow with fixed tab management"""
        print("🚀 Starting Warp Auto-Login (FIXED VERSION)")
        print("=" * 50)
        
        try:
            # Setup browser
            if not self.setup_driver():
                return False
            
            # Get temporary email (this creates the first tab)
            if not self.get_temp_email():
                return False
            
            print(f"\n📧 Email acquired: {self.temp_email}")
            print(f"📊 Current tabs: {len(self.driver.window_handles)}")
            
            # Open Warp in NEW tab while preserving email tab
            if not self.open_warp_in_new_tab():
                return False
            
            print(f"📊 After opening Warp: {len(self.driver.window_handles)} tabs")
            
            # Login to Warp
            if not self.login_to_warp():
                return False
            
            print("\n📧 Checking for verification email...")
            
            # Check email for verification (this will switch back to email tab)
            verification_link = self.check_email_for_verification(max_wait_minutes=3)
            
            if verification_link:
                print("✅ Verification link found!")
                
                # Complete Firebase auth
                if self.complete_firebase_auth(verification_link):
                    print("\n🎉 Authentication completed successfully!")
                    
                    # Keep browser open for manual token capture
                    print("\n🔍 Browser staying open for token capture...")
                    print("Please manually copy the auth token from Warp and press Enter...")
                    input()
                    
                    return True
                else:
                    print("❌ Firebase authentication failed")
            else:
                print("❌ No verification email received")
            
            # Keep browser open for manual completion
            print("\n🔍 Browser staying open for manual completion...")
            input("Press Enter to close browser...")
            
            return True
            
        except KeyboardInterrupt:
            print("\n⚠️  Process interrupted by user")
            return False
        except Exception as e:
            print(f"❌ Unexpected error: {e}")
            return False
        finally:
            self.cleanup()

def main():
    """Entry point for fixed version"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Fixed Warp terminal auto-login")
    parser.add_argument("--headless", action="store_true", help="Run in headless mode")
    args = parser.parse_args()
    
    # Run the fixed version
    warp_login = WarpAutoLoginFixed(headless=args.headless)
    success = warp_login.run()
    
    if success:
        print("\n✅ Process completed successfully!")
    else:
        print("\n❌ Process failed. Check the output above for details.")
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()
