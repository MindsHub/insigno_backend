from django.contrib.gis.db import models
from django.contrib.gis.geos import Point


class Marker(models.Model):
    xy = models.GeometryField(default=Point(45.0, 10.0, srid=4326))

    class TypeClass(models.TextChoices):
        UNKNOWN = 'unknown'
        PLASTIC = 'plastic'
        PAPER = 'paper'
        UNDIFFERENTIATED = 'undifferentiated'
        GLASS = "glass"
        COMPOST = "compost"
        ELECTRONICS = "electronics"

    type = models.CharField(
        max_length=20,
        choices=TypeClass.choices,
        default=TypeClass.UNDIFFERENTIATED,
    )

    def __str__(self):
        return f"xy =\"{self.xy}\""
