import datetime
import json

from django.contrib.gis.db.models.functions import Distance
from django.contrib.gis.geos import GEOSGeometry
from django.contrib.gis.measure import D
from django.http import JsonResponse, HttpResponse, HttpResponseNotAllowed
from .models import Marker, MarkerImage


# @ decorator
def post(function):
    def inner(*args, **kwargs):
        if args[0].method != "POST":
            return HttpResponseNotAllowed(["POST"])
        args = (json.loads(args[0].body), *args[1:])
        result = function(*args, **kwargs)
        return HttpResponse() if result is None else result

    return inner


# Create your views here.
def getNearMarkers(request, x, y):
    # pnt = GEOSGeometry('POINT(11.003322 45.755382)') #11.003322 45.755382 sede mindshub
    # print(f'POINT({x} {y})')
    pnt = GEOSGeometry(f'POINT({x} {y})', srid=4326)
    # found = Marker.objects.filter(xy__distance_lte=(pnt, D(km=10))).annotate(distance=Distance('xy', pnt)).order_by('distance') filtra anche i 10 km più vicini
    found = Marker.objects.annotate(distance=Distance('xy', pnt)).order_by('distance')
    arr = []
    for cur in found:
        punto = cur.xy
        arr.append(
            {"x": punto.x, "y": punto.y,
             'creationDate': cur.creationDate,
             'id': cur.pk,
             "type": cur.type,
             "distance": cur.distance.m}
        )
    return JsonResponse(arr, safe=False)


@post
def addMarkers(jsonData):
    cur = Marker(
        xy=GEOSGeometry(f"POINT({jsonData['x']} {jsonData['y']})", srid=4326),
        creationDate=datetime.timezone.now(),
        type=jsonData['time'],

    )
    cur.save()
    curImage = MarkerImage(
        marker_id=cur.pk,
        image=jsonData['image'],
    )
    cur.marker_id
    return JsonResponse({}, safe=False)
