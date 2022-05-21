from django.http import JsonResponse
from rest_framework.views import APIView;
from rest_framework.response import Response;
from rest_framework.permissions import IsAuthenticated;
import random

class HelloView(APIView):
    permissions_classes = [IsAuthenticated]

    def get(self, request):
        context = {"message": "Hello, World!"}
        return Response(context)