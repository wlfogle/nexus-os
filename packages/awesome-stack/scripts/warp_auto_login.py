#!/usr/bin/env python3
"""
Warp Terminal Auto-Login Script
Automates the process of:
1. Getting a fresh temporary email from 10minutemail
2. Logging into Warp at app.warp.dev/login/remote
3. Checking email for Firebase auth verification link
4. Completing authentication and capturing auth token
5. Passing token to Warp terminal application
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

class WarpAutoLogin:
    def __init__(self, headless=False):
        self.driver = None
        self.temp_email = None
        self.headless = headless
        
    def human_delay(self, min_seconds=1, max_seconds=3):
        """Add human-like random delay"""
        delay = random.uniform(min_seconds, max_seconds)
        time.sleep(delay)
        
    def type_like_human(self, element, text):
        """Type text with ultra-realistic human-like delays"""
        element.clear()
        # Sometimes pause before starting to type
        if random.random() < 0.3:
            time.sleep(random.uniform(0.5, 2.0))
        
        for i, char in enumerate(text):
            element.send_keys(char)
            # Variable delays based on character type and position
            if char in '.@':
                time.sleep(random.uniform(0.15, 0.4))  # Longer for special chars
            elif i > 0 and text[i-1] == text[i]:  # Repeated chars
                time.sleep(random.uniform(0.08, 0.2))
            else:
                time.sleep(random.uniform(0.05, 0.25))
            
            # Occasional longer pauses (thinking)
            if random.random() < 0.1:
                time.sleep(random.uniform(0.3, 0.8))
        
    def setup_driver(self):
        """Initialize Firefox webdriver with appropriate options"""
        from selenium.webdriver.firefox.options import Options
        
        options = Options()
        if self.headless:
            options.add_argument('--headless')
        
        # Maximum stealth options to avoid detection
        options.add_argument('--no-sandbox')
        options.add_argument('--disable-dev-shm-usage')
        options.add_argument('--disable-blink-features=AutomationControlled')
        options.add_argument('--disable-extensions')
        options.add_argument('--disable-plugins')
        options.add_argument('--disable-images')
        options.add_argument('--disable-javascript')
        options.set_preference('dom.webdriver.enabled', False)
        options.set_preference('useAutomationExtension', False)
        options.set_preference('marionette', False)
        options.set_preference('dom.webnotifications.enabled', False)
        options.set_preference('media.navigator.enabled', False)
        options.set_preference('network.http.sendRefererHeader', 0)
        options.set_preference('privacy.trackingprotection.enabled', True)
        options.set_preference('general.useragent.override', 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36')
        
        try:
            # Try FireDragon first (common in Garuda Linux)
            try:
                options.binary_location = '/usr/bin/firedragon'
                self.driver = webdriver.Firefox(executable_path=GeckoDriverManager().install(), options=options)
                self.driver.implicitly_wait(10)
                print("✅ Using FireDragon browser")
                return True
            except:
                # Fallback to regular Firefox
                options.binary_location = None  # Reset binary location
                self.driver = webdriver.Firefox(executable_path=GeckoDriverManager().install(), options=options)
                self.driver.implicitly_wait(10)
                print("✅ Using Firefox browser")
                return True
                
        except Exception as e:
            try:
                # Last fallback for system geckodriver
                self.driver = webdriver.Firefox(options=options)
                self.driver.implicitly_wait(10)
                return True
            except Exception as e2:
                print(f"❌ Failed to initialize browser driver: {e}")
                print(f"❌ Fallback also failed: {e2}")
                print("Make sure FireDragon/Firefox and geckodriver are installed:")
                print("  sudo pacman -S firedragon geckodriver")
                print("  OR sudo pacman -S firefox geckodriver")
                return False
    
    def get_temp_email(self):
        """Get a fresh temporary email from various providers"""
        
        # Only use the most reliable, least blocked providers
        email_providers = [
            {
                'name': 'Guerrilla Mail',
                'url': 'https://www.guerrillamail.com/',
                'email_selector': '#email-widget',
                'copy_selector': None
            },
            {
                'name': 'Maildrop',
                'url': 'https://maildrop.cc/',
                'email_selector': 'input[readonly]',
                'copy_selector': None
            },
            {
                'name': 'YOPmail',
                'url': 'https://yopmail.com/',
                'email_selector': '#login',
                'copy_selector': None
            }
        ]
        
        for provider in email_providers:
            print(f"🔄 Trying {provider['name']}...")
            
            try:
                print(f"🌐 Loading {provider['name']} website...")
                self.driver.get(provider['url'])
                # Add mouse movements to appear more human
                self.driver.execute_script("window.scrollTo(0, Math.floor(Math.random() * 500));")
                self.human_delay(3, 6)  # Reduced delay with mouse activity
                
                # Immediately try to get the email address to avoid missing updates
                print(f"⏳ Attempting to get email immediately...")
                email_element = self.driver.find_element(By.CSS_SELECTOR, provider['email_selector'])
                self.human_delay(0.5, 1)  # Shorter pause before reading email
                
                # Get email address with provider-specific handling
                if provider['name'] == 'Guerrilla Mail':
                    # Try multiple selectors for Guerrilla Mail email
                    try:
                        email_span = self.driver.find_element(By.CSS_SELECTOR, '#email-widget span')
                        self.temp_email = email_span.text
                    except:
                        try:
                            # Alternative selector
                            email_display = self.driver.find_element(By.ID, 'email-widget')
                            self.temp_email = email_display.text.strip()
                        except:
                            # Fallback - look for any email pattern on page
                            import re
                            page_text = self.driver.page_source
                            email_match = re.search(r'[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}', page_text)
                            if email_match:
                                self.temp_email = email_match.group(0)
                elif provider['name'] == 'YOPmail':
                    # Generate random email for YOPmail
                    random_name = ''.join(random.choices('abcdefghijklmnopqrstuvwxyz0123456789', k=8))
                    self.temp_email = f"{random_name}@yopmail.com"
                    # Enter the email to create the inbox
                    email_element.clear()
                    email_element.send_keys(random_name)
                    time.sleep(1)
                    check_btn = self.driver.find_element(By.CSS_SELECTOR, 'input[value="Check Inbox"]')
                    check_btn.click()
                    time.sleep(3)
                else:
                    self.temp_email = email_element.get_attribute("value") or email_element.text
                
                if self.temp_email and '@' in self.temp_email:
                    print(f"✅ Got temporary email from {provider['name']}: {self.temp_email}")
                    
                    # Copy email to clipboard for backup
                    pyperclip.copy(self.temp_email)
                    
                    return True
                else:
                    print(f"⚠️  Invalid email from {provider['name']}: {self.temp_email}")
                    continue
                    
            except Exception as e:
                print(f"⚠️  {provider['name']} failed: {e}")
                continue
        
        print("❌ All temporary email providers failed")
        return False
    
    def login_to_warp_logged_in(self):
        """Navigate to Warp logged_in page and enter email"""
        print("🔄 Navigating to Warp logged_in page...")
        
        try:
            self.driver.get("https://app.warp.dev/logged_in/remote")
            
            # Wait for page to load and email input to be available
            time.sleep(3)
            
            # Try multiple selectors for email input with correct Warp selectors
            email_selectors = [
                ".modal-container-input",
                "input[placeholder='Your email address']",
                ".auth-form-email-form input",
                "input[type='text'][placeholder*='email']",
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
            
            # Clear and enter email with human-like typing
            self.human_delay(0.5, 1.5)  # Pause before typing
            self.type_like_human(email_input, self.temp_email)
            print(f"✅ Entered email: {self.temp_email}")
            
            # Look for continue/submit button with correct Warp selectors
            button_selectors = [
                ".modal-container-button-full-width",
                "button.modal-container-button--primary",
                "button[type='submit']",
                "//button[contains(text(), 'Continue')]",
                ".auth-form-email-form button"
            ]
            
            continue_button = None
            for selector in button_selectors:
                try:
                    if ":contains(" in selector:
                        # XPath for text-based selection
                        text = selector.split(':contains(')[1].split(')')[0].strip("'\"")
                        xpath = f"//button[contains(text(), '{text}')]"
                        continue_button = self.driver.find_element(By.XPATH, xpath)
                    else:
                        continue_button = self.driver.find_element(By.CSS_SELECTOR, selector)
                    break
                except NoSuchElementException:
                    continue
            
            if continue_button:
                continue_button.click()
                print("✅ Clicked continue button")
            else:
                print("⚠️  Could not find continue button, trying Enter key")
                email_input.send_keys("\n")
            
            time.sleep(2)
            return True
            
        except Exception as e:
            print(f"❌ Error on logged_in page: {e}")
            return False
    
    def login_to_warp_main(self):
        """Navigate to main Warp login page and enter email"""
        print("🔄 Navigating to main Warp login page...")
        
        try:
            print("🌐 Loading Warp login page...")
            self.driver.get("https://app.warp.dev/login/remote")
            
            # Wait for page to load with faster timing
            # Simulate human browsing behavior
            self.driver.execute_script("window.scrollTo(0, 100);")
            self.human_delay(3, 6)  # Much faster page load wait
            self.driver.execute_script("window.scrollTo(0, 0);")
            self.human_delay(1, 2)  # Faster scroll back
            
            # Try to find email input with correct Warp selectors
            email_selectors = [
                ".modal-container-input",
                "input[placeholder='Your email address']",
                ".auth-form-email-form input",
                "input[type='text'][placeholder*='email']",
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
                print("❌ Could not find email input field on login page")
                return False
            
            # Enter email with human-like typing
            print("⌨️  Entering email address...")
            self.human_delay(0.5, 1.5)  # Faster typing start
            self.type_like_human(email_input, self.temp_email)
            print(f"✅ Entered email on login page: {self.temp_email}")
            self.human_delay(0.5, 1)  # Faster pause after typing
            
            # Look for login/continue button with correct Warp selectors
            button_selectors = [
                ".modal-container-button-full-width",
                "button.modal-container-button--primary",
                "button[type='submit']",
                "//button[contains(text(), 'Continue')]",
                ".auth-form-email-form button"
            ]
            
            login_button = None
            for selector in button_selectors:
                try:
                    if ":contains(" in selector:
                        text = selector.split(':contains(')[1].split(')')[0].strip("'\"")
                        xpath = f"//button[contains(text(), '{text}')]"
                        login_button = self.driver.find_element(By.XPATH, xpath)
                    else:
                        login_button = self.driver.find_element(By.CSS_SELECTOR, selector)
                    break
                except NoSuchElementException:
                    continue
            
            if login_button:
                print("🖱️  Clicking login button...")
                self.human_delay(0.5, 1)  # Faster click timing
                login_button.click()
                print("✅ Clicked login button")
            else:
                print("⚠️  Could not find login button, trying Enter key")
                email_input.send_keys("\n")
            
            # Wait for login process to complete
            self.human_delay(3, 5)  # Faster login completion wait
            return True
            
        except Exception as e:
            print(f"❌ Error on login page: {e}")
            return False
    
    def check_temp_email_for_verification(self, max_wait_minutes=5):
        """Check temporary email for Firebase verification link"""
        print(f"🔄 Checking temporary email for verification link (waiting up to {max_wait_minutes} minutes)...")
        
        # Switch back to the email tab (first tab)
        print(f"📱 Switching back to email tab...")
        print(f"📊 Available tabs: {len(self.driver.window_handles)}")
        
        # Check if we still have the email tab
        if len(self.driver.window_handles) == 0:
            print("❌ No tabs available!")
            return None
        
        # Switch to first tab (should be email)
        self.driver.switch_to.window(self.driver.window_handles[0])
        current_url = self.driver.current_url
        print(f"✅ Switched to email tab, current URL: {current_url}")
        
        # If the first tab isn't the email provider, try to find it
        if "guerrillamail" not in current_url.lower() and "tempmail" not in current_url.lower() and "yopmail" not in current_url.lower() and "maildrop" not in current_url.lower():
            print("⚠️  First tab doesn't seem to be email provider, checking other tabs...")
            for i, handle in enumerate(self.driver.window_handles):
                self.driver.switch_to.window(handle)
                url = self.driver.current_url
                if any(provider in url.lower() for provider in ["guerrillamail", "tempmail", "yopmail", "maildrop"]):
                    print(f"✅ Found email tab at index {i}: {url}")
                    break
            else:
                print("❌ Could not find email provider tab!")
                return None
        
        max_checks = max_wait_minutes * 6  # Check every 10 seconds
        
        for i in range(max_checks):
            try:
                # Refresh the email inbox (like a human checking email)
                print(f"🔄 Refreshing inbox (check #{i+1})...")
                self.driver.refresh()
                self.human_delay(2, 4)  # Faster refresh wait
                
                # Look for emails in inbox with multiple selectors, including your specific XPath
                email_selectors = [
                    "/html/body/div[4]/div/div[3]/div[2]/form/table/tbody/tr[1]/td[2]",  # Your specific XPath
                    "//table//tr[1]//td[2]",  # More flexible version of your XPath
                    "tr[onclick]",     # Guerrilla Mail table rows
                    ".email-item",     # General email items
                    ".mail_row",       # Some providers use this
                    "tbody tr",        # Table body rows
                    "#email_list tr"   # Specific email list
                ]
                
                email_element = None
                for selector in email_selectors:
                    try:
                        if selector.startswith('//'):
                            # XPath selector
                            found_elements = self.driver.find_elements(By.XPATH, selector)
                        elif selector.startswith('/html'):
                            # Full XPath selector
                            found_elements = self.driver.find_elements(By.XPATH, selector)
                        else:
                            # CSS selector
                            found_elements = self.driver.find_elements(By.CSS_SELECTOR, selector)
                        
                        if found_elements:
                            email_element = found_elements[0]
                            print(f"📧 Found email using selector: {selector}")
                            break
                    except Exception as e:
                        print(f"⚠️  Selector {selector} failed: {e}")
                        continue
                
                if email_element:
                    print(f"✅ Found email to click")
                    
                    # Click on the email element
                    print("📧 Clicking on email...")
                    self.human_delay(0.5, 1)  # Faster email click
                    
                    try:
                        # Try clicking with JavaScript first (more reliable)
                        self.driver.execute_script("arguments[0].click();", email_element)
                        print("✅ Clicked email with JavaScript")
                    except:
                        # Fallback to regular click
                        email_element.click()
                        print("✅ Clicked email with regular click")
                    
                    self.human_delay(1, 2)  # Faster email load wait
                    
                    # Ensure the email content is ready to be read
                    try:
                        email_content_element = WebDriverWait(self.driver, 10).until(
                            EC.presence_of_element_located((By.TAG_NAME, "body"))
                        )
                        email_content = email_content_element.text
                        print(f"📄 Email content loaded: {email_content[:200]}...")
                    except TimeoutException:
                        email_content = self.driver.page_source
                        print("📄 Using page source due to timeout")
                    
                    # Extract Firebase auth link with multiple patterns
                    firebase_patterns = [
                        r'https://[a-zA-Z0-9.-]+\.firebaseapp\.com/__/auth/action\?[^\s\"\'<>\)\]]+',
                        r'https://astral-field-294621\.firebaseapp\.com/__/auth/action\?[^\s\"\'<>\)\]]+',
                        r'https://[^\s\"\'<>\)\]]+firebaseapp[^\s\"\'<>\)\]]+auth[^\s\"\'<>\)\]]+',
                        r'https://[^\s\"\'<>\)\]]+warp[^\s\"\'<>\)\]]+auth[^\s\"\'<>\)\]]+'
                    ]
                    
                    verification_link = None
                    
                    # First try to find clickable links in the page
                    try:
                        clickable_links = self.driver.find_elements(By.XPATH, "//a[contains(@href, 'firebaseapp') or contains(@href, 'auth')]")
                        if clickable_links:
                            href = clickable_links[0].get_attribute('href')
                            if 'firebaseapp.com' in href and 'auth' in href:
                                verification_link = href
                                print(f"✅ Found Firebase verification link via clickable element: {verification_link[:100]}...")
                    except Exception as e:
                        print(f"⚠️  Could not find clickable links: {e}")
                    
                    # If no clickable link found, try text patterns
                    if not verification_link:
                        for pattern in firebase_patterns:
                            firebase_links = re.findall(pattern, email_content)
                            if firebase_links:
                                verification_link = firebase_links[0]
                                print(f"✅ Found Firebase verification link via regex: {verification_link[:100]}...")
                                break
                    
                    if verification_link:
                        return verification_link
                    else:
                        print("⚠️  Email found but no Firebase verification link detected")
                        print("🔍 Looking for any clickable links in email...")
                        
                        # Try clicking any available links that might be verification links  
                        try:
                            all_clickable_links = self.driver.find_elements(By.TAG_NAME, "a")
                            print(f"🔗 Found {len(all_clickable_links)} clickable link(s) in email")
                            
                            for i, link in enumerate(all_clickable_links[:3]):  # Try first 3 links
                                href = link.get_attribute('href') or ''
                                text = link.text.strip()
                                print(f"   Link {i+1}: {href[:60]}... (text: '{text}')")
                                
                                # If it looks like a verification link, try clicking it
                                if any(keyword in href.lower() for keyword in ['verify', 'confirm', 'auth', 'login', 'firebase']):
                                    print(f"🖱️  Attempting to click verification link {i+1}...")
                                    try:
                                        self.driver.execute_script("arguments[0].click();", link)
                                        self.human_delay(3, 5)
                                        # Check if we got redirected to a new page
                                        if len(self.driver.window_handles) > 2:  # New tab opened
                                            return href  # Return the href as verification link
                                    except Exception as click_e:
                                        print(f"⚠️  Failed to click link {i+1}: {click_e}")
                                        continue
                        except Exception as e:
                            print(f"⚠️  Error finding clickable links: {e}")
                        
                        # Fallback: try to find any links that might be the verification
                        all_links = re.findall(r'https://[^\s\"\'<>\)\]]+', email_content)
                        if all_links:
                            print(f"🔗 Found {len(all_links)} text link(s) in email:")
                            for link in all_links[:3]:  # Show first 3 links
                                print(f"   - {link[:80]}...")
                                # If it contains firebase or auth, try it
                                if 'firebase' in link.lower() or 'auth' in link.lower():
                                    print(f"🎯 Trying potential verification link: {link[:60]}...")
                                    return link
                
                if i < max_checks - 1:
                    wait_time = random.randint(15, 25)  # Reduced wait time for quicker checks
                    print(f"🔄 No verification email yet, waiting {wait_time} seconds before next check... ({i+1}/{max_checks})")
                    time.sleep(wait_time)
                
            except Exception as e:
                print(f"⚠️  Error checking email: {e}")
                time.sleep(10)
        
        print("❌ Timeout waiting for verification email")
        return None
    
    def complete_firebase_auth(self, verification_link):
        """Complete Firebase authentication using verification link"""
        print("🔄 Completing Firebase authentication...")
        
        try:
            # Store current tab handles before opening new tab
            initial_handles = self.driver.window_handles.copy()
            email_tab = initial_handles[0]  # Email tab should be first
            warp_tab = initial_handles[1] if len(initial_handles) > 1 else None  # Warp tab
            
            print(f"📊 Tab status before Firebase auth: {len(initial_handles)} tabs")
            print(f"   Email tab: {email_tab[:8]}...")
            if warp_tab:
                print(f"   Warp tab: {warp_tab[:8]}...")
            
            # Open verification link in new tab (don't replace current tab)
            print("🔄 Opening Firebase verification link in new tab...")
            self.driver.execute_script(f"window.open('{verification_link}', '_blank');")
            
            # Wait for new tab to open
            self.human_delay(2, 3)
            
            # Get updated handles
            updated_handles = self.driver.window_handles
            print(f"📊 Tab status after opening link: {len(updated_handles)} tabs")
            
            # Find the new Firebase auth tab (should be the last one)
            firebase_tab = None
            for handle in updated_handles:
                if handle not in initial_handles:
                    firebase_tab = handle
                    break
            
            if not firebase_tab:
                print("❌ Could not find Firebase auth tab")
                return False
            
            # Switch to Firebase auth tab
            print(f"🔄 Switching to Firebase auth tab: {firebase_tab[:8]}...")
            self.driver.switch_to.window(firebase_tab)
            
            # Wait for Firebase auth to complete
            self.human_delay(3, 5)
            
            # Look for success indicators or redirect
            current_url = self.driver.current_url
            print(f"✅ Firebase auth completed, current URL: {current_url[:60]}...")
            
            # Look for "Take me to Warp" button and click it
            if self.click_take_me_to_warp():
                print("✅ Successfully clicked 'Take me to Warp' button")
                self.human_delay(3, 5)  # Wait for page to load after click
                
                # Check if we're now on the logged_in page
                final_url = self.driver.current_url
                if "logged_in" in final_url or "app.warp.dev" in final_url:
                    print("✅ Successfully reached Warp logged_in page")
                    return True
            
            # Check if we're redirected to Warp logged_in page automatically
            if "app.warp.dev" in current_url:
                print("✅ Automatically redirected to Warp page")
                self.human_delay(3, 5)  # Wait for load completion
                return True
            
            # If Firebase auth completed but no redirect, navigate manually
            print("🔄 Navigating to logged_in page manually...")
            self.driver.get("https://app.warp.dev/logged_in/remote")
            self.human_delay(3, 5)
            return True
            
        except Exception as e:
            print(f"❌ Error completing Firebase auth: {e}")
            return False
    
    def click_take_me_to_warp(self):
        """Look for and click the 'Take me to Warp' button"""
        print("🔄 Looking for 'Take me to Warp' button...")
        
        try:
            # Wait a moment for page to fully load
            time.sleep(3)
            
            # Try multiple selectors for the button
            button_selectors = [
                "//button[contains(text(), 'Take me to Warp')]",
                "//a[contains(text(), 'Take me to Warp')]",
                "//button[contains(text(), 'take me to warp')]",
                "//a[contains(text(), 'take me to warp')]",
                "//button[contains(text(), 'Open Warp')]",
                "//a[contains(text(), 'Open Warp')]",
                "//button[contains(text(), 'Launch Warp')]",
                "//a[contains(text(), 'Launch Warp')]",
                ".warp-button",
                ".launch-button",
                "button[data-testid*='warp']",
                "a[data-testid*='warp']"
            ]
            
            for selector in button_selectors:
                try:
                    if selector.startswith('//'):
                        # XPath selector
                        button = WebDriverWait(self.driver, 5).until(
                            EC.element_to_be_clickable((By.XPATH, selector))
                        )
                    else:
                        # CSS selector
                        button = WebDriverWait(self.driver, 5).until(
                            EC.element_to_be_clickable((By.CSS_SELECTOR, selector))
                        )
                    
                    if button:
                        button.click()
                        print(f"✅ Clicked button with selector: {selector}")
                        time.sleep(2)
                        return True
                        
                except (TimeoutException, NoSuchElementException):
                    continue
            
            print("⚠️  Could not find 'Take me to Warp' button")
            return False
            
        except Exception as e:
            print(f"❌ Error clicking 'Take me to Warp' button: {e}")
            return False
    
    def capture_auth_token(self):
        """Capture the Warp authentication token from logged_in page"""
        print("🔄 Capturing Warp authentication token...")
        
        try:
            # Navigate to logged_in page if not already there
            if "logged_in" not in self.driver.current_url:
                self.driver.get("https://app.warp.dev/logged_in/remote")
                time.sleep(3)
            
            # Look for auth token in page content
            page_source = self.driver.page_source
            
            # Try to find token in various places
            token_patterns = [
                r'(warp://auth/desktop_redirect\?[^\s\"\'>]+)',  # Full auth URL
                r'"token"\s*:\s*"([^"]+)"',
                r'warp://login/([a-zA-Z0-9\-_]+)',
                r'refresh_token=([^&\s<>"\'>]+)'
            ]
            
            for pattern in token_patterns:
                matches = re.findall(pattern, page_source)
                if matches:
                    token = matches[0]
                    print(f"✅ Found auth token: {token[:20]}...{token[-10:]}")
                    
                    # Copy token to clipboard
                    pyperclip.copy(token)
                    print("✅ Token copied to clipboard")
                    
                    return token
            
            # Try to get token from local storage
            try:
                token = self.driver.execute_script(
                    "return localStorage.getItem('warp_auth_token') || "
                    "localStorage.getItem('authToken') || "
                    "localStorage.getItem('token')"
                )
                if token:
                    print(f"✅ Found auth token in localStorage: {token[:20]}...{token[-10:]}")
                    pyperclip.copy(token)
                    return token
            except:
                pass
            
            # Look for "here" link that contains the token
            try:
                here_links = self.driver.find_elements(By.XPATH, "//a[contains(text(), 'here')]")
                for link in here_links:
                    href = link.get_attribute('href')
                    if 'warp://' in href:
                        # Extract token from warp:// URL
                        token_match = re.search(r'warp://login/([a-zA-Z0-9\-_]+)', href)
                        if token_match:
                            token = token_match.group(1)
                            print(f"✅ Found auth token in 'here' link: {token[:20]}...{token[-10:]}")
                            pyperclip.copy(token)
                            return token
            except:
                pass
            
            print("❌ Could not find auth token")
            return None
            
        except Exception as e:
            print(f"❌ Error capturing auth token: {e}")
            return None
    
    def pass_token_to_warp(self, token):
        """Pass the auth token to Warp terminal"""
        print("🔄 Passing token to Warp terminal...")
        
        try:
            # Check if token is already a full URL or just a token
            if token.startswith('warp://'):
                warp_url = token
            else:
                # Fallback to old format if it's just a token
                warp_url = f"warp://login/{token}"
            
            print(f"🔗 Using auth URL: {warp_url[:50]}...")
            
            # Try opening with system default handler
            try:
                subprocess.run(["xdg-open", warp_url], check=True, timeout=10)
                print("✅ Opened Warp auth URL with system handler")
                return True
            except subprocess.CalledProcessError as e:
                print(f"⚠️  xdg-open failed: {e}")
            except Exception as e:
                print(f"⚠️  System handler failed: {e}")
            
            # Try launching Warp directly with full URL as argument
            warp_commands = [
                ["warp-terminal", warp_url],
                ["warp", warp_url],
                ["warp-terminal", "--auth-url", warp_url],
                ["warp", "--auth-url", warp_url]
            ]
            
            for cmd in warp_commands:
                try:
                    subprocess.run(cmd, check=True, timeout=10)
                    print(f"✅ Launched Warp with auth URL: {' '.join(cmd[:2])}")
                    return True
                except subprocess.CalledProcessError:
                    continue
                except Exception:
                    continue
            
            print("⚠️  Could not automatically pass auth URL to Warp")
            print(f"Manual steps:")
            print(f"1. Open Warp terminal manually")
            print(f"2. Use this auth URL: {warp_url}")
            print(f"3. Or copy the URL from clipboard and paste it in your browser")
            
            return False
            
        except Exception as e:
            print(f"❌ Error passing token to Warp: {e}")
            return False
    
    def open_warp_terminal(self):
        """Attempt to open warp-terminal application"""
        print("🔄 Opening Warp terminal application...")
        
        try:
            # Try different ways to open Warp terminal
            commands = [
                "warp-terminal",
                "warp",
                "/usr/bin/warp-terminal",
                "/opt/warp-terminal/warp-terminal"
            ]
            
            for cmd in commands:
                try:
                    result = subprocess.run([cmd], 
                                          capture_output=True, 
                                          text=True, 
                                          timeout=5)
                    if result.returncode == 0:
                        print(f"✅ Successfully launched Warp terminal with: {cmd}")
                        return True
                except (subprocess.TimeoutExpired, FileNotFoundError):
                    continue
            
            # If direct commands fail, try desktop file
            try:
                subprocess.run(["gtk-launch", "warp-terminal"], 
                             capture_output=True, timeout=5)
                print("✅ Launched Warp terminal via desktop file")
                return True
            except:
                pass
            
            print("⚠️  Could not automatically launch Warp terminal")
            print("Please manually open Warp terminal from your applications menu")
            return False
            
        except Exception as e:
            print(f"❌ Error opening Warp terminal: {e}")
            return False
    
    def cleanup(self):
        """Clean up resources"""
        if self.driver:
            print("🔄 Closing browser...")
            self.driver.quit()
    
    def run(self):
        """Main execution flow"""
        print("🚀 Starting Warp Auto-Login Process")
        print("=" * 50)
        
        try:
            # Setup browser
            if not self.setup_driver():
                return False
            
            # Get temporary email
            if not self.get_temp_email():
                return False
            
            # Open Warp login in new tab to avoid losing email tab
            print(f"🔄 Opening new tab for Warp login...")
            self.driver.execute_script("window.open('about:blank', '_blank');")
            self.human_delay(0.5, 1)  # Faster tab opening
            print(f"📱 Available tabs: {len(self.driver.window_handles)}")
            self.driver.switch_to.window(self.driver.window_handles[1])
            print(f"✅ Switched to Warp tab")
            
            # Login to main Warp page first (more reliable)
            if not self.login_to_warp_main():
                print("⚠️  Failed at main login page, trying logged_in page...")
                # Fallback to logged_in page
                if not self.login_to_warp_logged_in():
                    print("❌ Failed at both login pages")
                    return False
            
            print("\n📧 Waiting for verification email...")
            print(f"Email: {self.temp_email}")
            
            # Check for verification email automatically
            verification_link = self.check_temp_email_for_verification(max_wait_minutes=3)
            
            if verification_link:
                # Complete Firebase authentication
                if self.complete_firebase_auth(verification_link):
                    print("✅ Firebase authentication completed successfully")
                    
                    # Give some time for redirect and page load
                    time.sleep(5)
                    
                    # Capture auth token
                    auth_token = self.capture_auth_token()
                    
                    if auth_token:
                        print(f"\n🎉 Successfully captured auth token!")
                        
                        # Ask if user wants to automatically pass token to Warp
                        response = input("\nDo you want to automatically open Warp with this token? (y/n): ").lower()
                        if response in ['y', 'yes']:
                            if self.pass_token_to_warp(auth_token):
                                print("✅ Token passed to Warp successfully!")
                            else:
                                print("⚠️  Manual token entry required")
                        else:
                            print(f"\n📋 Auth token (copied to clipboard): {auth_token}")
                            print("You can paste this into Warp terminal when prompted.")
                        
                        return True
                    else:
                        print("❌ Could not capture auth token")
                        print("Please check the logged_in page manually for token")
                else:
                    print("❌ Firebase authentication failed")
            else:
                print("⚠️  No verification email received")
                print("Please check the email manually and complete verification")
            
            # Keep browser open for manual completion if needed
            print("\n🔍 Browser staying open for manual verification if needed...")
            input("Press Enter when you've completed the process manually to close browser...")
            
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
    """Entry point"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Automate Warp terminal login with temporary email")
    parser.add_argument("--headless", action="store_true", 
                       help="Run browser in headless mode")
    
    args = parser.parse_args()
    
    # Check dependencies
    try:
        import selenium
        import pyperclip
    except ImportError as e:
        print(f"❌ Missing required dependency: {e}")
        print("Install with: pip install selenium pyperclip")
        sys.exit(1)
    
    # Run the automation
    automation = WarpAutoLogin(headless=args.headless)
    success = automation.run()
    
    if success:
        print("\n✅ Process completed successfully!")
    else:
        print("\n❌ Process completed with errors")
        sys.exit(1)

if __name__ == "__main__":
    main()
