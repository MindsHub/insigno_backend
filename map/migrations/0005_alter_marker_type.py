# Generated by Django 4.0.5 on 2022-10-24 13:39

from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ('map', '0004_alter_marker_type'),
    ]

    operations = [
        migrations.AlterField(
            model_name='marker',
            name='type',
            field=models.CharField(choices=[('unknown', 'Unknown'), ('plastic', 'Plastic'), ('paper', 'Paper'), ('undifferentiated', 'Undifferentiated'), ('glass', 'Glass'), ('compost', 'Compost'), ('electronics', 'Electronics')], default='undifferentiated', max_length=20),
        ),
    ]