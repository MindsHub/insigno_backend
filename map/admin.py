from django.contrib.gis import admin

from map.models import Marker, MarkerImage

# Register your models here.
admin.site.register(Marker, admin.GISModelAdmin)
admin.site.register(MarkerImage)
