from django.db import models
from enum import Enum
from django.contrib.auth.models import User

class Pill(models.Model):
    text = models.CharField(max_length=200)

    def __str__(self):
        return self.text

class PillQuestion(models.Model):
    pill = models.ForeignKey(Pill, on_delete=models.CASCADE)
    question = models.CharField(max_length=200)
    is_multiple_choice = models.BooleanField()

    def __str__(self):
        return self.question + " - " + ("multiple choice" if self.is_multiple_choice else "single choice")

class PillAnswer(models.Model):
    pill_question = models.ForeignKey(PillQuestion, on_delete=models.CASCADE)
    answer = models.CharField(max_length=200)
    is_correct = models.BooleanField()

    def __str__(self):
        return self.answer + " - " + ("correct" if self.is_correct else "incorrect")

class InsignioUser(User):
    points = models.PositiveIntegerField()
    solved_pill_questions = models.ManyToManyField(Pill, blank=True)

    def __str__(self):
        return super(InsignioUser, self).__str__() + f" - points={self.points}"
