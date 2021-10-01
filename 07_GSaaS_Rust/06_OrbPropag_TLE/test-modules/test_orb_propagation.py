#!/usr/bin/env python3

"""
 (c) Incomplete Worlds 2021 
 Alberto Fernandez (ajfg)
  
 GS as a Service

 This script tests the Orbit Propagation using SGP4 / TLE
 * Login
 * Orbit Propagation TLE correct data
 * Orbit Propagation TLE incorrect data
 * Logout

 NOTE: They are executed in alphabetic order. So, they have a number before
 the name to ensure the order
"""


import unittest
import json
import requests
import hashlib

from datetime import timezone 
import datetime 

import sqlite3

unittest.TestLoader.sortTestMethodsUsing = None

API_SERVER_IP="http://127.0.0.1:9002/"
LOGIN_SERVER_IP="http://127.0.0.1:9000/"
user_id = ""
authentication_key = ""

class TestOrbPropagationTle(unittest.TestCase):
    # This setup is executed only one time
    @classmethod
    def setUpClass(cls):
        pass
        
        # cls.draft_register(cls)


    # def draft_register(self):
    #     global user_id

    #     print("Setup: Register user .....")

    #     api_url = LOGIN_SERVER_IP + "tools/register"

    #     with open('register_user_demo.json') as f:
    #         in_json = json.load(f)

    #     # print( json.dumps(in_json, indent = 4) )
    #     resp = requests.put(api_url,
    #                         headers={"content-type": "application/json"},
    #                         json=in_json)

    #     if resp.status_code != 200:
    #         print("Error registering user")
    #         print(resp.text)
    #         # self.assertFalse(True)
    #         return False

    #     else:
    #         resp_json = json.loads( resp.text )
    #         print(resp_json)

    #         # user_id = resp_json["user_id"]
    #         # print("user id = ", user_id)
    #         print("   Ok")





    def test_1login(self):
        global user_id, authentication_key

        print("Test Login  ......")
        api_url = LOGIN_SERVER_IP + "tools/login"
        
        with open('login.json') as f:
            in_json = json.load(f)

        # Getting the current date  
        # and time 
        dt = datetime.datetime.now() 
        
        utc_time = dt.replace(tzinfo = timezone.utc) 
        utc_timestamp = utc_time.timestamp() 
        
        print("     Time: ",int(utc_timestamp) )

        in_json["timestamp"] = int(utc_timestamp)

        # NOT NEEDED.   Password shall be hashed
        # clear_password = in_json["password"]
        # hashed_password = hashlib.sha256(clear_password.encode()).hexdigest();
        # # print(hashed_password)
        # in_json["password"] = hashed_password

        resp = requests.post(api_url,
                            headers={"content-type": "application/json"},
                            json=in_json)
       
        if resp.status_code != 200:
            print(resp.text)
            self.assertFalse(True)

        else:
            resp_json = json.loads( resp.text )

            print("user id   = ", resp_json["user_id"])
            print("jwt token = ", resp_json["jwt_token"])
            authentication_key = resp_json["jwt_token"]
            user_id = resp_json["user_id"]

            print("   Ok")


# ************************************************************************************
#              END OF TEST CASES
# ************************************************************************************



    def test_2orb_propagation_tle_correct(self):
        global user_id, authentication_key

        if authentication_key == None or authentication_key == "":
            print("User is not logged in. Abort")
            self.assertFalse(True)
            return

        print("Test Orbit Propagation TLE correct data .....")
        print("")

        api_url = API_SERVER_IP + "fdsaas/v1/orb_propagation_tle"

        with open('orb_propagation_tle.json') as f:
            in_json = json.load(f)


        # Getting the current date and time 
        dt = datetime.datetime.now() 
        
        utc_time = dt.replace(tzinfo = timezone.utc) 
        utc_timestamp = utc_time.timestamp() 
        
        # print( int(utc_timestamp) )

        in_json["timestamp"] = int(utc_timestamp)
        in_json["authentication_key"] = authentication_key
        in_json["user_id"] = user_id 


        # print( json.dumps(in_json, indent = 4) )
        resp = requests.get(api_url,
                            headers={"content-type": "application/json"},
                            json=in_json)

        if resp.status_code != 200:
            print("Error propagating orbit")
            print(resp.text)
            self.assertFalse(True)

        else:
            resp_json = json.loads( resp.text )
            # print(resp_json)
            print( json.dumps(resp_json, indent = 4) )

            print("   Ok")




# ************************************************************************************
#              END OF TEST CASES
# ************************************************************************************

    def test_99logout(self):
        global user_id, authentication_key
        
        print("Test Logout  ......")
        api_url = LOGIN_SERVER_IP + "tools/logout"

        if authentication_key == None or authentication_key == "":
            self.assertFalse(True)
            print("Authentication key is empty. Abort")
            return

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
        in_json["user_id"] = user_id 
        
        resp = requests.delete(api_url,
                            headers={"content-type": "application/json"},
                            json=in_json)

        if resp.status_code != 200:
            print(resp.text)
            self.assertFalse(True)

        else:
            resp_json = json.loads( resp.text )
            print("   Ok")




if __name__ == "__main__":
    unittest.main()
