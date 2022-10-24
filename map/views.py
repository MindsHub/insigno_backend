import json

from django.contrib.gis.db.models.functions import Distance
from django.contrib.gis.geos import GEOSGeometry
from django.contrib.gis.measure import D
from django.http import JsonResponse, HttpResponse
from .models import Marker


# Create your views here.
def getNearMarkers(request, x, y):
    #pnt = GEOSGeometry('POINT(11.003322 45.755382)') #11.003322 45.755382 sede mindshub
    #print(f'POINT({x} {y})')
    pnt = GEOSGeometry(f'POINT({x} {y})', srid=4326)
    #found = Marker.objects.filter(xy__distance_lte=(pnt, D(km=10))).annotate(distance=Distance('xy', pnt)).order_by('distance') filtra anche i 10 km più vicini
    found = Marker.objects.annotate(distance=Distance('xy', pnt)).order_by('distance')
    arr = []
    for cur in found:
        punto = cur.xy
        arr.append(
            {"x": punto.x, "y": punto.y, "type": cur.type, "distance": cur.distance.m}
        )
    return JsonResponse(arr, safe=False)
