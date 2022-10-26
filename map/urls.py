from django.urls import path

from . import views

urlpatterns = [
    path('getNearMarkers/<x>_<y>', views.getNearMarkers),
    path('addMarkers', views.addMarkers),

]