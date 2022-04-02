from django.http import JsonResponse
from .models import Pill
import random

def randomPill(request):
    pks = Pill.objects.values_list('pk', flat=True)
    random_pk = random.choice(pks)
    r = Pill.objects.get(pk=random_pk)
    return JsonResponse({
        'text': r.text,
        'author': r.author,
        'source': r.source,
    })
