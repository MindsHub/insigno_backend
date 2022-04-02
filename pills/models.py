from django.db import models

class Pill(models.Model):
    text = models.CharField(max_length=200)
    author = models.CharField(max_length=50)
    source = models.URLField(blank=True)

    def __str__(self):
        return f"\"{self.text}\" di {self.author} ({self.source})"
