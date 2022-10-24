from django.contrib.gis import admin

from map.models import Marker

# Register your models here.
admin.site.register(Marker, admin.GISModelAdmin)
