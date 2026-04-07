FROM python:3.11-slim

WORKDIR /app

# Install dependencies first (cached layer)
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy app source
COPY app.py .
COPY templates/ templates/

EXPOSE 8088

ENV PYTHONUNBUFFERED=1

CMD ["python", "app.py"]
