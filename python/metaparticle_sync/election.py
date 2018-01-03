import threading
from lock import Lock


class Election:
    def __init__(
                self, name, is_master_callback, lost_master_callback):
        self.lock = Lock(
            name, lock_callback=self._lock, lock_lost_callback=self._lost_lock)
        self.master_callback = is_master_callback
        self.lost_master_callback = lost_master_callback
        self.running = False
        self.condition = threading.Condition()

    def shutdown(self):
        self.running = False
        self.condition.acquire()
        self.condition.notify()
        self.condition.release()

    def run(self):
        self.running = True
        while self.running:
            self.lock.acquire()
            self.condition.acquire()
            self.condition.wait()
            self.condition.release()
        self.lock.release()

    def _lock(self):
        self.master_callback()

    def _lost_lock(self):
        self.lost_master_callback()
