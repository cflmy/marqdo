import re, unicodedata
ascii_lower = str.maketrans('ABCDEFGHIJKLMNOPQRSTUVWXYZ','abcdefghijklmnopqrstuvwxyz')
bridge = [{'id': row['id'], 'tokens': [t for t in re.split('[ \n\t\r\u00a0]+',unicodedata.normalize('NFC',row['text']).translate(ascii_lower)) if t]}
          for row in data]
bridge.sort(key=lambda row:row['id'].encode('utf-8'))
bridge
