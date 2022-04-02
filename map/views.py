from django.http import JsonResponse
from .models import Pill
import random

def randomPill(request):
    pks = Pill.objects.values_list('pk', flat=True)
    random_pk = random.choice(pks)
    return JsonResponse({
        'text': Pill.objects.get(pk=random_pk).text
    })
