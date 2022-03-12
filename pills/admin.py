from django.contrib import admin
from pills.models import Pill, PillQuestion, PillAnswer

admin.site.register(Pill)
admin.site.register(PillQuestion)
admin.site.register(PillAnswer)
