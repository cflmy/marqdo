#!/usr/bin/env python3
"""Rebuild the Marqdo Markdown → View → execution promotional film.

Source imagery:
  - assets/marketing/user-procurement-view.png: original user-provided View capture
  - assets/marketing/source-markdown-view-concept.png: original generated concept art
Other images are generated from the repository's actual procurement source.
The purchasing demo is simulated, not an ERP transaction.
"""
from pathlib import Path
import math, subprocess, wave, tempfile
import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageFilter, ImageOps
from markdown_it import MarkdownIt
from selenium import webdriver
from selenium.webdriver.chrome.options import Options

ROOT=Path(__file__).resolve().parents[1]
A=ROOT/"assets"/"marketing"
OUT=A/"marqdo-2.7.1-markdown-to-execution.mp4"
POSTER=A/"marqdo-2.7.1-markdown-to-execution-cover.jpg"
SRT=A/"marqdo-2.7.1-markdown-to-execution.srt"
W,H=1600,900
D=[4.8,6.8,6.4,7.0,7.5,9.5,8.8,5.2]
IVORY=(249,247,242);INK=(37,35,34);GOLD=(191,146,81);PEARL=(242,232,214);DIM=(122,114,105);NIGHT=(18,23,32)
REG="/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"
BOLD="/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc"
SERIF="/usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc"
SERIFB="/usr/share/fonts/opentype/noto/NotoSerifCJK-Bold.ttc"
def f(size,mode="regular"):
    return ImageFont.truetype({"regular":REG,"bold":BOLD,"serif":SERIF,"serifbold":SERIFB}[mode],size)
def txt(d,xy,s,size=26,color=INK,style="regular"):
    d.text(xy,str(s),font=f(size,style),fill=color)
def card(d,box,fill=IVORY,stroke=(220,210,195),r=24):
    d.rounded_rectangle(box,radius=r,fill=fill,outline=stroke,width=2)
def base(dark=False):
    im=Image.new("RGB",(W,H),NIGHT if dark else IVORY)
    d=ImageDraw.Draw(im)
    if dark:
        for j in range(8):
            d.arc((880+j*32,-200+j*26,1760+j*28,1030+j*25),158,268,fill=(68+j*7,60+j*6,51+j*4),width=2)
    else:
        for j in range(7):
            d.arc((-430+j*14,-500+j*14,500+j*25,520+j*25),35,150,fill=(238,224+j*2,207+j*2),width=2)
    return im
def rounded_image(im,src,box,focus=(.5,.5)):
    x1,y1,x2,y2=box
    crop=ImageOps.fit(src,(x2-x1,y2-y1),method=Image.Resampling.LANCZOS,centering=focus)
    mask=Image.new("L",crop.size,0)
    ImageDraw.Draw(mask).rounded_rectangle((0,0,crop.width-1,crop.height-1),radius=18,fill=255)
    im.paste(crop,(x1,y1),mask)
    ImageDraw.Draw(im).rounded_rectangle(box,radius=18,outline=(200,177,145),width=2)
def footer(d,text,dark=False):
    txt(d,(80,855),text,18,(148,158,171) if dark else DIM)
def make_intro():
    im=base(True);d=ImageDraw.Draw(im)
    txt(d,(94,113),"MARQDO   /   v2.7.1",21,(233,197,146))
    txt(d,(90,221),"Your SKILL",83,IVORY,"serifbold")
    txt(d,(90,327),"is now a",83,IVORY,"serifbold")
    txt(d,(90,435),"TOOL.",91,(244,201,148),"serifbold")
    d.line((100,562,1040,562),fill=GOLD,width=3)
    txt(d,(102,613),"不只是让 AI 阅读文档。",35,(232,230,224),"serif")
    txt(d,(102,681),"让同一份 Markdown 承载可控执行流。",35,(232,230,224),"serif")
    footer(d,"WRITE ONCE  ·  RENDER AS MARKDOWN  ·  INSPECT AS RUNTIME",True)
    return im

source=(ROOT/"examples/procurement/index.mq.md").read_text(encoding="utf8")
assert "## 查询采购商价格" in source
section=source.split("## 查询采购商价格",1)[1].split("\n## ",1)[0]
excerpt="## 查询采购商价格\n"+section
lines=excerpt.splitlines()
content=[]
for line in lines:
    if len(content)>=18:break
    content.append(line)
assert len(content)>10
md=MarkdownIt("commonmark").enable("table")
html=md.render("\n".join(content))
document=f"""<!DOCTYPE html><html lang='zh'><head><meta charset='utf-8'>
<style>html,body{{margin:0;background:#fcfbf6;color:#34302d}}
body{{font-family:'Noto Serif CJK SC',serif;padding:64px 96px;font-size:26px;line-height:1.75}}
h2{{font-size:43px;font-weight:700;margin:0 0 33px}}
p{{margin:17px 0}}table{{font:20px/1.7 'Noto Sans CJK SC',sans-serif;width:100%;border-collapse:collapse;margin:26px 0}}
td,th{{padding:12px 18px;border:1px solid #e1d7c8;text-align:left}}
th{{background:#efe7d9}}tr:nth-child(even){{background:#f7f2e9}}em{{color:#8c704f}}
</style></head><body>{html}</body></html>"""
opts=Options()
for arg in ["--headless=new","--no-sandbox","--disable-dev-shm-usage","--disable-gpu","--window-size=1450,1600"]:
    opts.add_argument(arg)
browser=webdriver.Chrome(options=opts)
try:
    browser.set_window_size(1450,1600)
    browser.get("data:text/html;charset=utf-8,"+__import__("urllib.parse",fromlist=["quote"]).quote(document))
    htmlscreen=A/"marqdo-procurement-standard-markdown.png"
    browser.save_screenshot(str(htmlscreen))
    assert htmlscreen.stat().st_size>20000
finally:
    browser.quit()
RENDER=Image.open(htmlscreen).convert("RGB")
USER=Image.open(A/"user-procurement-view.png").convert("RGB")
CONCEPT=Image.open(A/"source-markdown-view-concept.png").convert("RGB")

def make_source():
    im=base(True);d=ImageDraw.Draw(im)
    txt(d,(78,69),"01   /   SOURCE",21,(226,192,141))
    txt(d,(77,123),"源码态：它本来就是 Markdown",54,IVORY,"serifbold")
    txt(d,(80,209),"普通段落保持可读，标题、参数表和返回契约仍是 Markdown。",28,(187,198,213))
    card(d,(75,265,1520,797),(28,38,51),(78,89,105),23)
    card(d,(76,265,1519,332),(43,53,70),(43,53,70),23)
    for j,col in enumerate([(190,97,91),(224,173,97),(109,173,130)]):
        d.ellipse((105+j*33,293,122+j*33,310),fill=col)
    txt(d,(256,282),"index.mq.md   ·   查询采购商价格",24,(227,227,222))
    for i,line in enumerate(content[:16]):
        value=line[:77].replace("\t","    ")
        clr=(232,193,133) if value.startswith("#") or "返回" in value else ((164,214,201) if "正常" in value else (221,226,230))
        txt(d,(106,358+i*25),f"{i+1:02d}   {value}",20,clr)
    footer(d,"ACTUAL .mq.md EXCERPT  ·  source: examples/procurement/index.mq.md",True)
    return im

def make_render():
    im=base();d=ImageDraw.Draw(im)
    txt(d,(81,69),"02   /   STANDARD MARKDOWN",21,(170,126,67))
    txt(d,(77,128),"渲染态：它首先是一篇文档",56,INK,"serifbold")
    txt(d,(80,205),"同一段源码，无需定制语法高亮，也可以按 Markdown 阅读。",29,DIM)
    card(d,(80,265,1516,839),(255,254,251))
    rounded_image(im,RENDER,(120,290,1469,814),(.5,.19))
    txt(ImageDraw.Draw(im),(1145,280),"STANDARD MARKDOWN",18,(167,115,55))
    footer(ImageDraw.Draw(im),"A MARKDOWN-IT RENDER  ·  FROM THE SAME SOURCE")
    return im

def make_triple():
    im=base();d=ImageDraw.Draw(im)
    txt(d,(75,60),"03   /   THREE VIEWS, ONE FILE",21,(170,126,67))
    txt(d,(76,130),"同一份 .mq.md，三种视图",55,INK,"serifbold")
    txt(d,(81,211),"源码能编写，Markdown 能阅读，View 能看到函数、值和执行依赖。",27,DIM)
    screens=[("SOURCE  /  源码",make_source()),("MARKDOWN  /  渲染",RENDER),("VIEW  /  执行",USER)]
    xlist=[75,578,1081]
    for i,(label,src) in enumerate(screens):
        x=xlist[i];tw=442
        card(d,(x,280,x+tw,777),(255,254,250))
        txt(d,(x+21,301),label,23,(130,95,56))
        rounded_image(im,src,(x+15,349,x+tw-15,749),(.50,.38))
        d=ImageDraw.Draw(im)
    for x in [523,1026]:
        d.line((x,530,x+49,530),fill=GOLD,width=4)
        d.polygon([(x+49,530),(x+32,519),(x+32,541)],fill=GOLD)
    footer(d,"NOT THREE FILES  ·  ONE DOCUMENT + EXECUTION SEMANTICS")
    return im

def make_concept():
    im=base();d=ImageDraw.Draw(im)
    txt(d,(72,58),"04   /   DESIGN LANGUAGE",21,(170,126,67))
    rounded_image(im,CONCEPT,(55,112,1545,850),(.5,.5))
    txt(ImageDraw.Draw(im),(92,855),"CONCEPT ART  ·  Source → Rendered Markdown → Marqdo View",18,DIM)
    return im

def make_view():
    im=base();d=ImageDraw.Draw(im)
    txt(d,(78,62),"05   /   REAL MARQDO VIEW",21,(170,126,67))
    txt(d,(78,127),"在文档上，看见它的执行流",58,INK,"serifbold")
    txt(d,(80,216),"自然语言依然是主体，执行信息沿优雅的曲线引出。",28,DIM)
    card(d,(70,278,1530,841),(255,254,250))
    rounded_image(im,USER,(88,295,1513,817),(.50,.50))
    footer(ImageDraw.Draw(im),"ACTUAL VIEW SCREENSHOT  ·  PROVIDED BY THE PROJECT AUTHOR")
    return im

def make_execution():
    im=base(True);d=ImageDraw.Draw(im)
    txt(d,(80,66),"06   /   EXECUTABLE FUNCTIONS",21,(230,192,137))
    txt(d,(80,132),"在哪里执行？就在同一份文档的函数里。",53,IVORY,"serifbold")
    txt(d,(84,216),"说明是可读的，规则却会通过函数调用实际检查。",28,(183,198,211))
    card(d,(76,275,1519,367),(30,42,57),(78,88,100))
    txt(d,(99,303),"marqdo call examples/procurement/index.mq.md 查询余额 已花费=120",25,(233,218,188))
    card(d,(76,418,760,775),(25,55,48),(70,124,96))
    card(d,(790,418,1520,775),(58,34,43),(154,85,96))
    txt(d,(110,455),"PASS   /   LOCAL SIMULATION",24,(150,227,185))
    txt(d,(822,455),"DENIED   /   BUDGET CHECK",24,(249,164,170))
    txt(d,(112,533),"¥120 + ¥288",43,IVORY)
    txt(d,(823,533),"¥360 + ¥288",43,IVORY)
    txt(d,(113,606),"¥408",86,(146,229,185),"bold")
    txt(d,(823,606),"¥648",86,(250,162,166),"bold")
    txt(d,(115,720),"ok: true  ·  本地模拟，未发送",23,(217,237,224))
    txt(d,(824,720),"ok: false  ·  超预算拒绝",23,(248,216,217))
    footer(d,"OUTCOMES VERIFIED WITH MARQDO 2.7.1 CLI  ·  NO REAL ORDER WAS SENT",True)
    return im

def make_outro():
    im=base(True);d=ImageDraw.Draw(im)
    txt(d,(97,157),"Marqdo",90,IVORY,"serifbold")
    txt(d,(98,313),"Write it.  Read it.  Run it.",61,(238,195,136),"serifbold")
    d.line((99,431,1480,431),fill=GOLD,width=3)
    txt(d,(101,500),"A document-native runtime for executable AI skills.",31,(217,221,225))
    txt(d,(101,649),"github.com/cflmy/marqdo",43,IVORY)
    footer(d,"Marqdo v2.7.1  ·  Apache 2.0  ·  simulated procurement demo",True)
    return im

slides=[make_intro(),make_source(),make_render(),make_triple(),make_concept(),make_view(),make_execution(),make_outro()]
A.mkdir(parents=True,exist_ok=True)
slides[0].save(POSTER,quality=95)
with tempfile.TemporaryDirectory() as tmp:
    tmp=Path(tmp);clips=[]
    for i,(still,dur) in enumerate(zip(slides,D)):
        imagefile=tmp/f"slide_{i:02d}.png"
        still.save(imagefile)
        clip=tmp/f"clip_{i:02d}.mp4"
        filt=f"scale=1280:720,zoompan=z='min(zoom+0.00016,1.02)':x='iw/2-iw/zoom/2':y='ih/2-ih/zoom/2':d=1:s=1280x720:fps=24,fade=t=in:st=0:d=0.4,fade=t=out:st={dur-.5}:d=0.5,format=yuv420p"
        cmd=["ffmpeg","-y","-hide_banner","-loglevel","error","-loop","1","-framerate","24","-t",str(dur),"-i",str(imagefile),"-vf",filt,"-an","-c:v","libx264","-preset","veryfast","-crf","19","-pix_fmt","yuv420p",str(clip)]
        subprocess.run(cmd,check=True,timeout=200)
        clips.append(clip)
        print("RENDERED_SLIDE",i,flush=True)
    playlist=tmp/"concat.txt"
    playlist.write_text("\n".join(f"file '{c}'" for c in clips)+"\n")
    silent=tmp/"silent.mp4"
    subprocess.run(["ffmpeg","-y","-hide_banner","-loglevel","error","-f","concat","-safe","0","-i",str(playlist),"-c","copy",str(silent)],check=True,timeout=90)
    sr=24000
    total=sum(D);ts=np.arange(int(total*sr))/sr
    sound=np.zeros(len(ts))
    at=0.
    for i,dur in enumerate(D):
        mask=(ts>=at)&(ts<at+dur);tt=ts[mask]-at
        a=np.minimum(np.minimum(tt/.8,(dur-tt)/.8),1).clip(0,1)
        note=[110.,123.47,130.81,146.83,110.,123.47,130.81,110.][i]
        sound[mask]+=a*.12*(.55*np.sin(2*np.pi*note*tt)+.2*np.sin(2*np.pi*note*1.5*tt)+.12*np.sin(2*np.pi*note*2*tt))
        at+=dur
    sound*=np.minimum(np.minimum(ts/1.2,(total-ts)/1.2),1).clip(0,1)
    wav=tmp/"ambience.wav"
    with wave.open(str(wav),"wb") as w:
        w.setnchannels(1);w.setsampwidth(2);w.setframerate(sr)
        w.writeframes((sound*32767).astype(np.int16).tobytes())
    subprocess.run(["ffmpeg","-y","-hide_banner","-loglevel","error","-i",str(silent),"-i",str(wav),"-c:v","copy","-c:a","aac","-b:a","160k","-shortest","-movflags","+faststart",str(OUT)],check=True,timeout=90)

times=[0,4.8,11.6,18,25,32.5,42,50.8,56]
messages=[
 "Your SKILL is now a TOOL.\n让 Markdown 直接承载可执行语义。",
 "源码态：标题、段落、表格都沿用 Markdown。\n执行逻辑就在文档之中。",
 "渲染态：同一份代码也能成为普通 Markdown 文档。",
 "源码态、标准 Markdown 渲染态、Marqdo View 态。\n三种视图，同一份源文件。",
 "一份 .mq.md，既可以阅读，也可以编译执行。",
 "真实 View 截图：函数调用和值沿着优雅曲线呈现。",
 "预算校验不是文字承诺，而是函数执行。\n模拟订单通过或拒绝，均由运行时返回结果。",
 "Marqdo 2.7.1  ·  github.com/cflmy/marqdo"
]
def stamp(s):
    ms=int(round(s*1000))
    h=ms//3600000;ms%=3600000;m=ms//60000;ms%=60000
    return f"{h:02d}:{m:02d}:{ms//1000:02d},{ms%1000:03d}"
SRT.write_text("\n\n".join(f"{i+1}\n{stamp(times[i])} --> {stamp(times[i+1])}\n{msg}" for i,msg in enumerate(messages))+"\n",encoding="utf-8")
print("VIDEO",OUT,OUT.stat().st_size)
print("COVER",POSTER,POSTER.stat().st_size)
print("SRT",SRT,SRT.stat().st_size)
assert OUT.stat().st_size>800000 and sum(D)==56
