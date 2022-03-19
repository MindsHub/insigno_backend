from django.urls import path

from . import views

urlpatterns = [
    path('random', views.randomPill, name='random pill'),
]