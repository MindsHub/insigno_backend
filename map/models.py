from django.db import models

class Marker(models.Model):
    x = models.FloatField()
    y = models.FloatField()

    class TypeClass(models.TextChoices):
        PLASTIC = 'Plastic'
        PAPER = 'Paper'
        UNDIFFERENTIATED = 'Undifferentiated'
        GLASS = "Glass"
        COMPOST = "Compost"
        ELECTRONICS = "Electronics"

    type = models.CharField(
        max_length=20,
        choices=TypeClass.choices,
        default=TypeClass.UNDIFFERENTIATED,
    )


    def __str__(self):
        return f"x =\"{self.x}\" y = {self.y}"
