import csv, io
from decimal import Decimal
bridge = [{'quality':int(row['quality']), 'density_ppm':int(Decimal(row['density'])*1000000), 'ph_cent':int(Decimal(row['pH'])*100)}
          for row in csv.DictReader(io.StringIO(data),delimiter=';')]
bridge
