FROM python:3.12-slim
WORKDIR /repo
COPY bindings/heeczer-py /repo/bindings/heeczer-py
COPY testing/compose/py/chatbot /app
COPY testing/tests/fixtures /fixtures
RUN pip install --no-cache-dir /repo/bindings/heeczer-py fastapi 'uvicorn[standard]'
WORKDIR /app
EXPOSE 8000
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8000"]
