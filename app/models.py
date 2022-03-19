from django.db import models
from enum import Enum
from django.contrib.auth.models import User
from pills.models import Pill
import datetime

class InsignioUser(User):
    points = models.PositiveIntegerField(default=0)
    timesOpenedApp = models.PositiveIntegerField(default=0)
    lastOpenedApp = models.DateTimeField(auto_now=True)

    def __str__(self):
        return super(InsignioUser, self).__str__() + f" - points={self.points}, timesOpenedApp={self.timesOpenedApp}, lastOpenedApp={self.lastOpenedApp}"
