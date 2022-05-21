from django.urls import path
from map.views import HelloView

from . import views

urlpatterns = [
    path("hello", HelloView.as_view()),
]