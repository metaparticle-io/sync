from six.moves import urllib
import os
import sys
import threading
import time

class Lock:
    def __init__(self, name, base_uri='http://localhost:13131', lock_callback=None, lock_lost_callback=None):
        self.name = name
        self.base_uri = base_uri
        self.maintainer = None
        self.lock = threading.Lock()
        self.running = False
        self.lock_callback=lock_callback
        self.lock_lost_callback=lock_lost_callback

    def acquire(self, blocking=True):
        if not self.lock.acquire(blocking):
            return False
        if self.maintainer is not None:
            self.lock.release()
            raise threading.ThreadError()

        if not blocking:
            result = self._acquire_lock()
            self.lock.release()
            return result

        while not self._acquire_lock():
            time.sleep(10)

        self.lock.release()
        return True

    def release(self):
        self.lock.acquire()
        if self.maintainer is None:
            self.lock.release()
            raise threading.ThreadError()

        self.running = False
        self.maintainer.join()
        self.lock.release()

    def _acquire_lock(self):
        code = -1
        try:
            code = self._get_lock()
            if code == 404 or code == 200:
                code = self._update_lock()
            if code == 200:
                self._hold_lock()
                if self.lock_callback is not None:
                    self.lock_callback()
                return True
        except:
            print('Unexpected error: {}'.format(sys.exc_info()[0]))
            raise
        return False

    def _hold_lock(self):
        self.running = True
        self.maintainer = threading.Thread(target=self._run)
        self.maintainer.start()

    def _run(self):
        while self.running:
            code = self._get_lock()
            if code == 200:
                code = self._update_lock()
            if code != 200:
                print('Unexpected status: {}'.format(code))
                if self.lost_lock_callback is not None:
                    self.lost_lock_callback()
                    break
                else:
                    os.exit(-1)
            time.sleep(10)
        self.maintainer = None

    def _get_lock(self):
        req = urllib.request.Request(self.base_uri + '/locks/' + self.name)
        try:
            res = urllib.request.urlopen(req)
            return res.getcode()
        except urllib.error.HTTPError as ex:
            return ex.code

    def _update_lock(self):
        req = urllib.request.Request(self.base_uri + '/locks/' + self.name, headers={ 'accept': 'application/json' })
        req.get_method = lambda: 'PUT'
        try:
            res = urllib.request.urlopen(req)
            return res.getcode()
        except urllib.error.HTTPError as ex:
            return ex.code


