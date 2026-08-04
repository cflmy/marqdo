# Marqdo Spike（Python）

快速验收 ADR 0001 中的 S1–S5，**不是**完整解释器。

```bash
cd spike
python -m venv .venv
# Windows:
.\.venv\Scripts\activate
pip install -r requirements.txt
pytest -q
```

若需代理：`pip install -r requirements.txt --proxy http://127.0.0.1:7897`
