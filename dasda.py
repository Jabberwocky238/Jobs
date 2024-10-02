import abc
import os
from time import sleep

class Base:
    def __init__(self):
        self.name = "Base"
        self.age = 0

    @abc.abstractmethod
    def run(self):
        print("Base run")

class Derived(Base):
    def __init__(self):
        super().__init__()
        self.name = "Derived"
        self.age = 10

    def run(self):
        super().run()


a = Derived()
while True:
    a.run()
    sleep(0.1)  # 每隔1秒执行一次