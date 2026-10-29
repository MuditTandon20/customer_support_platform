FROM python:3.12-slim-trixie
RUN apt-get update && apt-get upgrade -y && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY requirements.txt requirements.lock ./
RUN pip install --no-cache-dir --timeout 120 --retries 5 pip==26.2.2 \
    && pip install --no-cache-dir --timeout 120 --retries 5 -r requirements.txt \
    && python -m pip uninstall -y pip && rm -rf /usr/local/lib/python3.12/ensurepip \
    && useradd -m app
COPY app ./app
RUN find / -xdev -type f -perm /6000 -exec chmod a-s {} +
ENV PYTHONDONTWRITEBYTECODE=1
USER app
EXPOSE 8001
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "8001"]
