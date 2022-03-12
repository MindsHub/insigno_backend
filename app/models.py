from django.db import models
from enum import Enum
from django.contrib.auth.models import User
from pills.models import Pill

class InsignioUser(User):
    points = models.PositiveIntegerField()
    solved_pill_questions = models.ManyToManyField(Pill, blank=True)

    def __str__(self):
        return super(InsignioUser, self).__str__() + f" - points={self.points}"
