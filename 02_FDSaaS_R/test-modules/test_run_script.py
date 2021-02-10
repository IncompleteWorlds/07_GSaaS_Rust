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


class TestRunScript(unittest.TestCase):
    # This setup is executed only one time
    @classmethod
    def setUpClass(cls):
        cls.do_login(cls)

    def do_login(self):
        global user_id, authentication_key
        
        api_url = API_SERVER_IP + "fdsaas/api/login"

        with open('login.json') as f:
            in_json = json.load(f)

        # Getting the current date  
        # and time 
        dt = datetime.datetime.now() 
        
        utc_time = dt.replace(tzinfo = timezone.utc) 
        utc_timestamp = utc_time.timestamp() 
        
        # print(int(utc_timestamp) )

        in_json["timestamp"] = int(utc_timestamp)

        clear_password = in_json["password"]
        hashed_password = hashlib.sha256(clear_password.encode()).hexdigest();
        # print(hashed_password)
        in_json["password"] = hashed_password

        resp = requests.get(api_url,
                            headers={"content-type": "application/json"},
                            json=in_json)
       
        print("Login response status code: ", resp.status_code)
        if resp.status_code != 200:
            print("ERROR: Unable to login; ", resp.text)
            exit(0)

        resp_json = json.loads( resp.text )

        value = resp_json.get('error')
        if value:
            print(value)
            print("ERROR: Unable to login; ", resp.text)
            exit(0)
        else:
            print("user id   = ", resp_json["user_id"])
            print("jwt token = ", resp_json["jwt_token"])
            user_id = resp_json["user_id"]
            authentication_key = resp_json["jwt_token"]
            print("   Ok")


    def test_run(self):
        global user_id, authentication_key

        print("Test Run Script  ......")
        api_url = API_SERVER_IP + "fdsaas/api/run_script"
        
        with open('run_script.json') as f:
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
       
        self.assertEqual(resp.status_code, 200)

        print(resp.text)
        # resp_json = json.loads( resp.text )

        # value = resp_json.get('error')
        # if value:
        #     print(value)
        #     self.assertFalse()
        # else:

        #     print("   Ok")

    def test_run_usage(self):
        global user_id, authentication_key

        print("Test Run Script Usage ......")
        api_url = API_SERVER_IP + "fdsaas/api/run_script/usage"
        
        resp = requests.get(api_url,
                            headers={"content-type": "application/json"})
       
        self.assertEqual(resp.status_code, 200)

        print(resp.text)



if __name__ == "__main__":
    unittest.main()
