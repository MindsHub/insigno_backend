from django import forms
from .models import *


class MarkerForm(forms.ModelForm):
    class Meta:
        model = Marker, MarkerImage
        fields = ['name', 'x', "y", 'image']