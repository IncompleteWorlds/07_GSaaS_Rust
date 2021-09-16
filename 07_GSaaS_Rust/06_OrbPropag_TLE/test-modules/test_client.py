#!/usr/bin/env python3

"""
 (c) Incomplete Worlds 2021 
 Alberto Fernandez (ajfg)
  
 GS as a Service

 This script tests the basic functionality of the Service
 * GetStatus
 * GetVersion
 * Exit 

 NOTE: They are executed in alphabetic order. So, exit() -? zexit
"""

import unittest
import json
import requests
import hashlib

from datetime import timezone 
import datetime 

import sqlite3

unittest.TestLoader.sortTestMethodsUsing = None

API_SERVER_IP="http://127.0.0.1:9000/"
user_id = ""
authentication_key = ""

# json.dumps(['foo', {'bar': ('baz', None, 1.0, 2)}])
# json.loads('["foo", {"bar":["baz", null, 1.0, 2]}]')

# def as_complex(dct):
# ...     if '__complex__' in dct:
# ...         return complex(dct['real'], dct['imag'])
# ...     return dct
# ...
# >>> json.loads('{"__complex__": true, "real": 1, "imag": 2}',
# ...     object_hook=as_complex)
# (1+2j)
# >>> import decimal
# >>> json.loads('1.1', parse_float=decimal.Decimal)
# Decimal('1.1')



# pload = {'username':'Olivia','password':'123'}
# r = requests.post('https://httpbin.org/post',data = pload)



class TestGeneral(unittest.TestCase):
    def test_version(self):
        print("Test Version  ......")
        api_url = API_SERVER_IP + "tools/version"
        
        resp = requests.get(api_url,
                            headers={"content-type": "application/json"},
                            data=None)

        resp_json = json.loads( resp.text )
        
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp_json["version"], '0.1')
        print("   Ok")


    def test_status(self):
        print("Test Status  ......")
        api_url = API_SERVER_IP + "tools/status"
        
        resp = requests.get(api_url,
                            headers={"content-type": "application/json"},
                            data=None)

        resp_json = json.loads( resp.text )
       
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp_json["status"], 'Running')
        # self.assertEqual(resp.text, '"Running"')
        print("   Ok")
        

    def test_zexit(self):
        print("Test Exit  ......")
        api_url = API_SERVER_IP + "tools/exit/XYZZY"
        
        resp = requests.post(api_url,
                            headers={"content-type": "application/json"},
                            data=None)
       
        # Status = No content
        self.assertEqual(resp.status_code, 204)
        # self.assertEqual(resp_json["status"], 'Running')
        # self.assertEqual(resp.text, '"Running"')
        print("   Ok")




if __name__ == "__main__":
    unittest.main()




