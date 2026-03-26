FROM python:3.11-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

EXPOSE 9900

CMD ["gunicorn", "-b", "0.0.0.0:9900", "-w", "2", "--threads", "4", "app:app"]
