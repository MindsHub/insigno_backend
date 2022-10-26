from django.contrib.gis.db import models
from django.contrib.gis.geos import Point


class Marker(models.Model):
    xy = models.GeometryField(default=Point(45.0, 10.0, srid=4326), null=False)

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
        null=False
    )

    creationDate = models.DateTimeField(null=False)

    def __str__(self):
        return f"(\"{self.pk}\")xy =\"{self.xy}\""


class MarkerImage(models.Model):
    image = models.ImageField()
    marker_id = models.ForeignKey(
        Marker,
        on_delete=models.CASCADE,
        null=False
    )

    def __str__(self):
        return f"(\"{self.pk}\")"
