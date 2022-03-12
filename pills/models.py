from django.db import models

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
