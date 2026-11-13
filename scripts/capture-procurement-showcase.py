#!/usr/bin/env python3
"""Reproducible marketing captures from the actual Marqdo 2.7.1 Linux CLI.

All procurement outputs are simulation-only. The GIF is a curated display of
the real function-call JSON responses, not a filmed human typing session.
The screenshot is taken with a real headless browser from marqdo view output.
"""
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "marketing"
SRC = "examples/procurement/index.mq.md"

def invoke(cli, fn, args=(), effects=None):
    cmd = [str(cli), "call", SRC, fn, *args]
    if effects:
        cmd += ["--effects", effects]
    cmd.append("--json")
    p = subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, timeout=45)
    if p.returncode != 0:
        raise RuntimeError("Command failed:\n" + " ".join(cmd) + "\n" + p.stdout + "\n" + p.stderr)
    try:
        result = json.loads(p.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(f"Non-JSON CLI response {p.stdout!r}") from exc
    if not result.get("ok"):
        raise RuntimeError("Marqdo call rejected: " + p.stdout)
    print("VERIFIED", fn, json.dumps(result, ensure_ascii=False))
    return result["result"]

def font(size, bold=False):
    for name in (
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc" if bold else "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ):
        try:
            return ImageFont.truetype(name, size)
        except OSError:
            continue
    return ImageFont.load_default()

def frame(index, headline, invocation, details, emphasis=None, danger=False):
    W, H = 1200, 660
    img = Image.new("RGB", (W,H), (14,22,37))
    d = ImageDraw.Draw(img)
    gold = (238,201,145); muted = (159,178,204)
    d.rounded_rectangle((24,22,1175,635), radius=25, fill=(22,32,51), outline=(59,77,101), width=2)
    d.text((53,49), "Marqdo", font=font(30,True), fill=gold)
    d.text((231,57), "EXECUTABLE SKILLS  /  PROCUREMENT",font=font(19),fill=muted)
    d.rounded_rectangle((976,44,1138,82), radius=16, fill=(39,54,75))
    d.text((1000,54),"v2.7.1",font=font(18),fill=(216,227,239))
    d.text((53,139),headline,font=font(32,True),fill=(245,244,239))
    d.text((54,188),"CALL",font=font(16,True),fill=gold)
    d.rounded_rectangle((49,208,1150,293),radius=11,fill=(12,19,33))
    for i,line in enumerate(invocation):
        d.text((66,223+i*34),line,font=font(19),fill=(194,221,240))
    d.text((54,336),"ACTUAL CLI RESULT  /  DOMAIN FIELDS",font=font(16,True),fill=gold)
    d.rounded_rectangle((49,352,1150,536),radius=11,fill=(16,26,43))
    for i,line in enumerate(details[:5]):
        d.text((69,371+i*33),line,font=font(20),fill=(219,233,241))
    if emphasis:
        accent=(245,153,159) if danger else (155,231,185)
        d.rounded_rectangle((52,554,1149,609),radius=10,fill=(68,38,48) if danger else (30,67,57))
        d.text((71,569),emphasis,font=font(22,True),fill=accent)
    for i in range(5):
        x=79+i*219
        d.rounded_rectangle((x,621,x+204,628),radius=3,fill=gold if i<=index else (69,81,99))
    d.text((855,641),f"FRAME {index+1} / 5  ·  SIMULATED ONLY",font=font(13),fill=muted)
    return img

def make_gif(cli):
    suppliers=invoke(cli,"查询采购商",["采购商状态='正常'"])
    quote=invoke(cli,"查询采购商价格",["采购商标识符='SUP_JN'","数量=8","有效秒数=300"])
    balance=invoke(cli,"查询余额",["已花费=120"])
    orderargs=["采购商标识符='SUP_JN'","数量=8","单价=36","交付要求天数=7"]
    good=invoke(cli,"向采购商发工单",orderargs+["已花费=120"],effects="time.read,process.control")
    bad=invoke(cli,"向采购商发工单",orderargs+["已花费=360"],effects="time.read,process.control")
    assert isinstance(suppliers,list) and any(x.get("标识符")=="SUP_JN" for x in suppliers), suppliers
    assert quote["单价"]==36 and quote["报价总额"]==288,quote
    assert balance["可用余额"]==480,balance
    assert good["ok"] is True and good["value"]["工单金额"]==288 and good["value"]["状态"]=="本地模拟，未发送",good
    assert bad["ok"] is False and "采购金额超过" in bad["error"],bad

    frames=[
        frame(0,"01  /  Supplier approval",
          ["$ marqdo call ... 查询采购商", "  采购商状态='正常'"],
          [f"result: {len(suppliers)} approved suppliers", "SUP_JN  /  江南配件  /  正常", "Paused and blacklisted suppliers excluded."],
          "✓  SUP_JN passes supplier status"),
        frame(1,"02  /  Quote lookup",
          ["$ marqdo call ... 查询采购商价格", "  采购商标识符='SUP_JN' 数量=8 有效秒数=300"],
          [f"单价: ¥{quote['单价']}",f"数量: {quote['数量']}",f"报价总额: ¥{quote['报价总额']}","报价来源: 示例目录（未接实时接口）"],
          "✓  ¥36 × 8 = ¥288 — example quote"),
        frame(2,"03  /  Budget check",
          ["$ marqdo call ... 查询余额 已花费=120"],
          [f"最高预算: ¥{balance['最高预算']}",f"已花费: ¥{balance['已花费']}",f"可用余额: ¥{balance['可用余额']}","还能下单: "+str(balance["还能下单"])],
          "✓  ¥480 ≥ ¥288 — budget check passes"),
        frame(3,"04  /  Callable simulated order",
          ["$ marqdo call ... 向采购商发工单", "  SUP_JN  数量=8  单价=36  已花费=120"],
          [f"result.ok: {str(good['ok']).lower()}", f"工单金额: ¥{good['value']['工单金额']}",f"状态: {good['value']['状态']}","No ERP message, payment or purchase was made."],
          "✓  SIMULATED ORDER ONLY — NOT SENT"),
        frame(4,"05  /  Over-budget request denied",
          ["$ marqdo call ... 向采购商发工单", "  SUP_JN  数量=8  单价=36  已花费=360"],
          [f"result.ok: {str(bad['ok']).lower()}","最高预算: ¥600","剩余额度: ¥240","工单金额: ¥288", "错误: "+bad["error"]],
          "×  WORK ORDER REFUSED AT RUNTIME",danger=True)
    ]
    OUT.mkdir(parents=True,exist_ok=True)
    name=OUT/"procurement-cli.gif"
    frames[0].save(name,save_all=True,append_images=frames[1:],duration=[1600,1600,1600,1900,2300],loop=0,optimize=True)
    assert name.stat().st_size>10000, name
    print("GIF_WRITTEN",name,name.stat().st_size)

def make_screenshot(cli):
    from selenium import webdriver
    from selenium.webdriver.common.by import By
    from selenium.webdriver.chrome.options import Options
    import time
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        dest=Path(td)/"view"
        cmd=[str(cli),"view","output","examples/procurement","--out",str(dest),"--no-exec"]
        p=subprocess.run(cmd,cwd=ROOT,text=True,capture_output=True,timeout=90)
        if p.returncode:
            raise RuntimeError("View output error:\n"+p.stdout+"\n"+p.stderr)
        pages=sorted(dest.rglob("*.html"),key=lambda p:len(p.parts))
        print("VIEW_PAGES",[str(p.relative_to(dest)) for p in pages[:40]])
        if not pages:
            raise RuntimeError("No HTML pages emitted by Marqdo View")
        chosen=next((p for p in pages if "向采购商发工单" in p.read_text(encoding="utf-8",errors="replace")),pages[0])
        opts=Options()
        for arg in ["--headless=new","--no-sandbox","--disable-dev-shm-usage","--disable-gpu","--window-size=1500,1000"]:
            opts.add_argument(arg)
        driver=webdriver.Chrome(options=opts)
        try:
            driver.set_window_size(1500,1000)
            driver.get(chosen.as_uri())
            time.sleep(2)
            headings=driver.find_elements(By.CSS_SELECTOR,"h1,h2,h3,h4")
            matches=[el for el in headings if "向采购商发工单" in el.text]
            print("VIEW_HEADINGS",[(el.tag_name,el.text[:65]) for el in headings[:25]])
            if not matches:
                raise RuntimeError("Couldn't find procurement function heading in exported View")
            target=matches[0]
            driver.execute_script("arguments[0].scrollIntoView({block:'start'}); window.scrollBy(0,-115);",target)
            time.sleep(1)
            shot=OUT/"procurement-view.png"
            driver.save_screenshot(str(shot))
            assert shot.stat().st_size>20000,shot
            print("VIEW_SCREENSHOT_WRITTEN",shot,shot.stat().st_size)
        finally:
            driver.quit()

if __name__=="__main__":
    a=argparse.ArgumentParser()
    a.add_argument("--cli",required=True)
    args=a.parse_args()
    make_gif(Path(args.cli).resolve())
    make_screenshot(Path(args.cli).resolve())
