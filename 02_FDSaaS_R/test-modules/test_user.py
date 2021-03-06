#!/usr/bin/env python3

"""
 (c) Incomplete Worlds 2020 
 Alberto Fernandez (ajfg)
  
 FDS as a Service 
"""

import unittest
import json
import requests
import hashlib

from datetime import timezone 
import datetime 

import sqlite3

unittest.TestLoader.sortTestMethodsUsing = None

API_SERVER_IP="http://127.0.0.1:11005/"
user_id = ""
authentication_key = ""

class TestUser(unittest.TestCase):
    # This setup is executed only one time
    @classmethod
    def setUpClass(cls):
        # Remove previuos user
        conn = sqlite3.connect('../data/fdsaas.db')
        conn.execute("DELETE FROM t_user WHERE username = 'user01'")
        conn.commit()
        conn.close()

        cls.draft_register(cls)

    def draft_register(self):
        global user_id

        print("Register user .....")

        api_url = API_SERVER_IP + "fdsaas/api/register"

        with open('register_user_demo.json') as f:
            in_json = json.load(f)

        # print( json.dumps(in_json, indent = 4) )
        resp = requests.put(api_url,
                            headers={"content-type": "application/json"},
                            json=in_json)

        if resp.status_code != 200:
            print("Error registering user")
            return

        # print(resp.text)
        resp_json = json.loads( resp.text )

        value = resp_json.get('error')
        if value:
            print(value)
        
        else:
            # res1 = json.loads( resp_json["response"])
            user_id = resp_json["user_id"]
            print("user id = ", user_id)
            print("   Ok")


    def test_login(self):
        global user_id, authentication_key

        print("Test Login  ......")
        api_url = API_SERVER_IP + "fdsaas/api/login"
        
        with open('login.json') as f:
            in_json = json.load(f)

        # Getting the current date  
        # and time 
        dt = datetime.datetime.now() 
        
        utc_time = dt.replace(tzinfo = timezone.utc) 
        utc_timestamp = utc_time.timestamp() 
        
        print( int(utc_timestamp) )

        in_json["timestamp"] = int(utc_timestamp)

        clear_password = in_json["password"]
        hashed_password = hashlib.sha256(clear_password.encode()).hexdigest();
        # print(hashed_password)
        in_json["password"] = hashed_password

        resp = requests.get(api_url,
                            headers={"content-type": "application/json"},
                            json=in_json)
       
        self.assertEqual(resp.status_code, 200)

        resp_json = json.loads( resp.text )

        value = resp_json.get('error')
        if value:
            print(value)
            self.assertFalse(True)
        else:
            print("user id   = ", resp_json["user_id"])
            print("jwt token = ", resp_json["jwt_token"])
            authentication_key = resp_json["jwt_token"]

            print("   Ok")


    def test_logout(self):
        global user_id, authentication_key
        
        print("Test Logout  ......")
        api_url = API_SERVER_IP + "fdsaas/api/logout"

        with open('logout.json') as f:
            in_json = json.load(f)

        # Getting the current date  
        # and time 
        dt = datetime.datetime.now() 
        
        utc_time = dt.replace(tzinfo = timezone.utc) 
        utc_timestamp = utc_time.timestamp() 
        
        print( int(utc_timestamp) )

        in_json["timestamp"] = int(utc_timestamp)
        in_json["authentication_key"] = authentication_key
        
        resp = requests.get(api_url,
                            headers={"content-type": "application/json"},
                            json=in_json)
        print(resp.text)
        resp_json = json.loads( resp.text )

        self.assertEqual(resp.status_code, 200)
        print("   Ok")



if __name__ == "__main__":
    unittest.main()
