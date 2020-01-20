#!/usr/bin/env python2

import re
import sys
import pytz
import json
import urllib
import sqlite3
import hashlib
import datetime

# adapted from https://gist.github.com/SegFaultAX/05e0f76a8dd5dd5d28964585f2b14049
def with_match(name, re):
    return r'(?P<%s>%s)' % (name, re)

def quoted(re):
    return r'"%s"' % re

def bracketed(re):
    return r'\[%s\]' % re

IP_RE = r"\d+(?:\.\d+){3}"
QUOTED_STRING_RE = r'[^\"]+'
TIMESTAMP_RE = r'[^\]]+'
IP_LIST_RE = r'(?:{ip}(?:\,\s+{ip})*)|-'.format(ip=IP_RE)
NUMERAL_RE = r'\d+(?:\.\d+)?'
LOG_PARTS = [
    with_match('ip', IP_RE),
    r'-',
    with_match('user', r'[^ ]*'),
    bracketed(with_match('ts', TIMESTAMP_RE)),
    quoted(with_match('request', QUOTED_STRING_RE)),
    with_match('resp_code', r'\d+'),
    with_match('resp_size', r'\d+|-'),
    quoted(with_match('referer', QUOTED_STRING_RE)),
    quoted(with_match('user_agent', QUOTED_STRING_RE)),
]
LOG_RE_RAW = r"^%s$" % "\s+".join(LOG_PARTS)
LOG_RE = re.compile(LOG_RE_RAW)

def main():
    db = sqlite3.connect('logs.sqlite3')
    cur = db.cursor()

    for line in sys.stdin:
        match = LOG_RE.match(line.strip())
        if not match:
            raise Exception('could not match %s with %s' % (line, LOG_RE_RAW))
        cols = match.groupdict()

        cols['method'], cols['path'], cols['protocol'] = cols['request'].split(' ')

        ts_str, offset = cols['ts'].rsplit(' ', 1)
        ts = datetime.datetime.strptime(ts_str, '%d/%b/%Y:%H:%M:%S')
        if offset == '+0000':
            ts = ts.replace(tzinfo=pytz.utc)
        else:
            raise Exception('unknown UTC offset: %s' % offset)

        cols['ts_utc'] = ts.astimezone(pytz.utc).strftime('%Y-%m-%d %H:%M:%S')

        if cols['resp_size'] == '-':
            cols['resp_size'] = 0

        cur.execute("""
            INSERT OR IGNORE
            INTO requests (ip, user, ts, method, path, protocol, resp_code, resp_size, referer, user_agent)
            VALUES (:ip, :user, :ts_utc, :method, :path, :protocol, :resp_code, :resp_size, :referer, :user_agent)
        """, cols)

    results = cur.execute("""
        SELECT ts, ip, path, user_agent, rowid
        FROM requests
        WHERE
            path LIKE '/_static/prest-v%.exe'
            AND NOT EXISTS (
                SELECT * FROM downloads
                WHERE request_id = requests.rowid
            )
            AND resp_code = 200
    """).fetchall()

    for ts, ip, path, user_agent, request_id in results:
        ip_info = json.load(
            urllib.urlopen('http://ipinfo.io/' + ip)
        )

        hostname = ip_info.get('hostname', '')
        pieces = hostname.split('.')

        if all(len(s) <= 3 for s in pieces[-2:]):
            domain = '.'.join(pieces[-3:])
        else:
            domain = '.'.join(pieces[-2:])

        match = re.match(r'/_static/prest-v(.*).exe', path)
        if not match:
            raise Exception('could not extract version')
        version = match.group(1)

        country = ip_info.get('country')
        city = ip_info.get('city')
        user_agent_hash = hashlib.sha256(user_agent).hexdigest()[:8]

        cur.execute("""
            INSERT INTO downloads (ts, version, country, city, domain, user_agent_hash, request_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
        """, (ts, version, country, city, domain, user_agent_hash, request_id))

        print('logged download: %s' % request_id)

    db.commit()
    db.close()

if __name__ == '__main__':
    main()
