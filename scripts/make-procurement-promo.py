#!/usr/bin/env python3
"""Make a 16:9 Marqdo promotional video from checked-in repository assets.

Inputs: GIF generated from actual v2.7.1 CLI results, and screenshot captured
from actual v2.7.1 View. The budget scenario is explicitly simulated.
Everything used to reproduce this movie is in the repository.
No third-party soundtrack or music clips are used.
"""
from __future__ import annotations

import math
import os
import subprocess
import tempfile
import wave
from bisect import bisect_right
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont, ImageOps

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets" / "marketing"
GIF = ASSETS / "procurement-cli.gif"
VIEW = ASSETS / "procurement-view.png"
PROC = ROOT / "examples" / "procurement" / "index.mq.md"
OUT = ASSETS / "marqdo-2.7.1-promo.mp4"
POSTER = ASSETS / "marqdo-2.7.1-poster.jpg"
SUBTITLES = ASSETS / "marqdo-2.7.1-promo.srt"
W, H, FPS = 1280, 720, 24
DURATION = 32.0
CUTS = [0.0, 3.9, 9.0, 17.4, 23.4, 29.0, DURATION]
DARK = (11, 19, 32)
GOLD = (232, 187, 123)
LIGHT = (248, 246, 238)
MUTED = (155, 174, 197)
GREEN = (125, 224, 174)
RED = (251, 151, 155)

def rgb(c):
    return tuple(c)

def choose_font(size, bold=False, serif=False):
    paths = [
        "/usr/share/fonts/opentype/noto/NotoSerifCJK-Bold.ttc" if serif and bold else "",
        "/usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc" if serif else "",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Bold.ttc" if bold else "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf" if bold else "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ]
    for p in paths:
        if not p:
            continue
        try:
            return ImageFont.truetype(p, size)
        except OSError:
            continue
    return ImageFont.load_default()

F = {
    "hero": choose_font(72, True),
    "big": choose_font(55, True),
    "h1": choose_font(42, True),
    "h2": choose_font(31, True),
    "mid": choose_font(25, True),
    "code": choose_font(23),
    "body": choose_font(22),
    "small": choose_font(17),
    "micro": choose_font(14),
    "zh": choose_font(28, serif=True),
}

def text(draw, xy, msg, font, color=LIGHT, anchor=None):
    draw.text(xy, str(msg), font=font, fill=color, anchor=anchor)

def panel(draw, xy, fill=(24,35,54), outline=(51,67,89), rad=23, width=2):
    draw.rounded_rectangle(xy, radius=rad, fill=fill, outline=outline, width=width)

def mix(a, b, t):
    return a + (b-a)*t

def ease(x):
    u = max(0., min(1., x))
    return u*u*(3-2*u)

def appear(t, delay=0., span=.65):
    return ease((t-delay)/span)

def xshift(t, delay=.0):
    return int(38*(1-appear(t,delay)))

def opacity_overlay(base, overlay, alpha):
    return Image.blend(base, overlay, max(0.,min(1.,alpha)))

def clipped(src, size, center=(.5,.5), zoom=1):
    w,h=size
    if zoom > 1:
        p=ImageOps.fit(src,(int(w*zoom),int(h*zoom)),method=Image.Resampling.LANCZOS,centering=center)
        z=(p.width-w)//2
        v=(p.height-h)//2
        return p.crop((z,v,z+w,v+h))
    return ImageOps.fit(src,size,method=Image.Resampling.LANCZOS,centering=center)

def background(seed=0):
    im = Image.new("RGB",(W,H),DARK)
    pix=im.load()
    for y in range(H):
        for x in range(W):
            glow= max(0,1-math.hypot((x-1090)/1.3,(y-95)/1.1)/520)
            edge= max(0,1-math.hypot((x-25),y-690)/500)
            pix[x,y]=(int(11+14*glow+7*edge),int(19+9*glow+7*edge),int(32+13*glow+10*edge))
    d=ImageDraw.Draw(im)
    for x in range(45,W,82):
        d.line((x,0,x,H),fill=(18,29,45),width=1)
    for y in range(48,H,82):
        d.line((0,y,W,y),fill=(18,29,45),width=1)
    return im

BG=background()

def base_frame(t,scene):
    im=BG.copy()
    d=ImageDraw.Draw(im)
    # top identity, bottom running progress
    text(d,(63,35),"MARQDO",F["small"],GOLD)
    text(d,(168,36),"/  EXECUTABLE AI SKILLS",F["micro"],MUTED)
    d.rounded_rectangle((1079,21,1211,57),radius=15,fill=(40,51,68))
    text(d,(1095,30),"v2.7.1",F["small"],LIGHT)
    d.line((62,76,1214,76),fill=(48,61,80),width=2)
    d.line((62,694,1214,694),fill=(47,61,80),width=2)
    d.line((62,694,62+int((1152)*(t/DURATION)),694),fill=GOLD,width=4)
    text(d,(64,705),"DOCUMENT  →  FUNCTION  →  TOOL",F["micro"],(104,120,141))
    text(d,(1115,705),"%02d / 06" % (scene+1),F["micro"],(104,120,141))
    return im,d

def scene_hero(local,global_t):
    im,d=base_frame(global_t,0)
    a=appear(local,.07)
    yy=xshift(local,.08)
    text(d,(69,167+yy),"Your SKILL",F["hero"],LIGHT)
    text(d,(69,264+yy),"is now a",F["hero"],LIGHT)
    text(d,(69,361+yy),"TOOL.",F["hero"],GOLD)
    # typographic bar
    x=69
    d.rounded_rectangle((x,485,x+min(643,int(643*a)),489),radius=2,fill=GOLD)
    text(d,(71,520),"Write it.  Inspect it.  Call it.",F["mid"],LIGHT)
    text(d,(71,573),"让可读文档中的函数，真正成为可调用工具。",F["zh"],(224,219,205))
    d.rounded_rectangle((975,207,1169,493),radius=25,fill=(26,37,57),outline=(84,99,118),width=2)
    text(d,(1017,276),".mq.md",F["h2"],GOLD)
    text(d,(1017,357),"01   Read",F["small"],MUTED)
    text(d,(1017,399),"02   Call",F["small"],MUTED)
    text(d,(1017,441),"03   Check",F["small"],MUTED)
    return im

SOURCE_LINES=[
 "## 向采购商发工单",
 "采购商标识符   数量   单价   已花费",
 "",
 "正常商户 = 查询采购商(...)",
 "报价 = 查询采购商价格(...)",
 "预算 = 查询余额(已花费=...)",
 "",
 "1. 剩余额度 < 本单金额",
 "   → 拒绝模拟工单",
]
def scene_source(local,global_t):
    im,d=base_frame(global_t,1)
    text(d,(71,110),"ONE DOCUMENT.",F["h1"],LIGHT)
    text(d,(71,159),"Contract, data, policy and implementation in the same .mq.md.",F["body"],MUTED)
    panel(d,(67,205,842,642),fill=(18,28,44),outline=(72,91,113))
    d.rounded_rectangle((67,205,842,254),radius=17,fill=(35,48,67))
    text(d,(93,218),"index.mq.md     /     procurement skill",F["small"],GOLD)
    ready=int(len(SOURCE_LINES)*ease((local-.40)/3.8))
    for i,ln in enumerate(SOURCE_LINES[:ready]):
        clr=GOLD if i in (0,7) else GREEN if i==8 else (178,204,232)
        text(d,(101,279+i*37),ln,F["code"],clr)
    panel(d,(863,205,1214,642),fill=(24,35,55),outline=(74,88,107))
    for j,(label,desc) in enumerate([
        ("01  READ","human-readable"),
        ("02  CHECK","typed contracts"),
        ("03  CALL","marqdo call"),
        ("04  RETURN","structured values")
    ]):
        y=268+j*96
        d.rounded_rectangle((888,y,1187,y+66),radius=13,fill=(30,48,65))
        text(d,(907,y+11),label,F["small"],GOLD)
        text(d,(907,y+34),desc,F["micro"],(186,203,220))
    text(d,(71,662),"Excerpt simplified for screen layout; the full source is linked in the repository.",F["micro"],(110,135,154))
    return im

GIF_FRAMES=[]
GIF_DURATIONS=[]
if GIF.exists():
    im=Image.open(GIF)
    for i in range(getattr(im,"n_frames",1)):
        im.seek(i)
        GIF_FRAMES.append(im.convert("RGB").copy())
        GIF_DURATIONS.append(max(.3, im.info.get("duration",1600)/1000))
    if len(GIF_FRAMES) < 5:
        raise RuntimeError(f"Expected >= 5 real CLI demonstration frames, found {len(GIF_FRAMES)}")
else:
    raise FileNotFoundError(GIF)
GIF_CUM=[0.]
for dur in GIF_DURATIONS:
    GIF_CUM.append(GIF_CUM[-1]+dur)
GIF_CYCLE=GIF_CUM[-1]
GIF_FITTED=[clipped(im,(1136,620)) for im in GIF_FRAMES]

def scene_cli(local,global_t):
    im,d=base_frame(global_t,2)
    # The 5 panels are constructed from real calls and validated in the original capture script.
    k=min(len(GIF_FITTED)-1,bisect_right(GIF_CUM,(local+.3)%GIF_CYCLE)-1)
    im.paste(GIF_FITTED[k],(72,73))
    d=ImageDraw.Draw(im)
    d.rectangle((73,73,1210,113),fill=(15,24,37))
    text(d,(91,82),"REAL CLI RETURNS  /  CURATED ANIMATED WALKTHROUGH",F["small"],GOLD)
    d.rectangle((76,664,1206,688),fill=(15,24,37))
    text(d,(87,665),"Supplier → Quote → Balance → Mock order → Over-budget refusal",F["small"],LIGHT)
    return im

def scene_outcomes(local,global_t):
    im,d=base_frame(global_t,3)
    text(d,(70,113),"THE LIMIT IS EXECUTED.",F["h1"])
    text(d,(70,164),"Two calls. Same unit price. Different accumulated spend.",F["body"],MUTED)
    q=ease((local-.6)/1.5)
    left=120+int(288*q)
    right=360+int(288*q)
    panel(d,(70,219,616,610),fill=(17,49,43),outline=(69,124,102))
    panel(d,(652,219,1202,610),fill=(55,29,39),outline=(150,78,90))
    text(d,(99,253),"ALLOWED · SIMULATION",F["small"],GREEN)
    text(d,(681,253),"DENIED · RUNTIME CHECK",F["small"],RED)
    text(d,(99,304),"¥120  +  ¥288",F["h2"],LIGHT)
    text(d,(681,304),"¥360  +  ¥288",F["h2"],LIGHT)
    text(d,(99,396),"¥"+str(left),F["hero"],GREEN)
    text(d,(681,396),"¥"+str(right),F["hero"],RED)
    text(d,(103,445),"≤  ¥600",F["mid"],(196,239,212))
    text(d,(686,445),">  ¥600",F["mid"],(255,202,211))
    d.line((100,476,581,476),fill=(77,119,101),width=2)
    d.line((681,476,1170,476),fill=(135,80,90),width=2)
    text(d,(101,497),"ok: true",F["mid"],GREEN)
    text(d,(680,497),"ok: false",F["mid"],RED)
    text(d,(101,553),"本地模拟，未发送",F["body"],(211,233,220))
    text(d,(680,553),"采购金额超过剩余额度",F["body"],(251,213,217))
    text(d,(71,651),"The function re-checks its own budget constraints. This is an offline example, not a real purchase.",F["micro"],MUTED)
    return im

VIEW_IMAGE=Image.open(VIEW).convert("RGB")
VIEW_PANEL=(101,195,1179,635)
VIEW_MASK=Image.new("L",(VIEW_PANEL[2]-VIEW_PANEL[0],VIEW_PANEL[3]-VIEW_PANEL[1]),0)
ImageDraw.Draw(VIEW_MASK).rounded_rectangle((0,0,VIEW_MASK.width-1,VIEW_MASK.height-1),radius=18,fill=255)

def scene_view(local,global_t):
    im,d=base_frame(global_t,4)
    text(d,(70,111),"THE SAME SOURCE, BEAUTIFULLY INSPECTABLE.",F["h2"])
    text(d,(71,158),"Marqdo 2.7.1 View — an actual screenshot of the procurement function.",F["body"],MUTED)
    factor=1+.06*ease(local/5)
    content=clipped(VIEW_IMAGE,(VIEW_MASK.width,VIEW_MASK.height),center=(.50,.38),zoom=factor)
    im.paste(content,(VIEW_PANEL[0],VIEW_PANEL[1]),VIEW_MASK)
    d=ImageDraw.Draw(im)
    d.rounded_rectangle(VIEW_PANEL,radius=18,outline=(199,165,118),width=3)
    text(d,(104,665),"From marqdo view output • function 向采购商发工单",F["small"],(187,207,222))
    return im

def scene_outro(local,global_t):
    im,d=base_frame(global_t,5)
    text(d,(71,163),"One .mq.md",F["hero"],LIGHT)
    text(d,(71,260),"Many ways to use it.",F["h1"],GOLD)
    d.rounded_rectangle((70,365,1212,436),radius=19,fill=(33,49,67),outline=(81,101,124),width=2)
    text(d,(107,381),"Write it.   Inspect it.   Call it.",F["h2"],LIGHT)
    text(d,(73,490),"github.com/cflmy/marqdo",F["h1"],GOLD)
    text(d,(73,548),"Marqdo 2.7.1  •  Rust-based document-native runtime",F["body"],MUTED)
    text(d,(73,613),"Demo uses simulated supplier data. No ERP connection or actual purchase.",F["small"],(176,190,204))
    return im

SCENE_FNS=[scene_hero,scene_source,scene_cli,scene_outcomes,scene_view,scene_outro]
def render(t):
    i=min(len(SCENE_FNS)-1,bisect_right(CUTS,t)-1)
    im=SCENE_FNS[i](t-CUTS[i],t)
    end=CUTS[i+1]
    if i<len(SCENE_FNS)-1 and t > end-.43:
        other=SCENE_FNS[i+1](0,t)
        im=Image.blend(im,other,ease((t-(end-.43))/.43))
    return im

def make_audio(path):
    sr=24000
    count=round(DURATION*sr)
    times=np.arange(count,dtype=np.float64)/sr
    signal=np.zeros(count,dtype=np.float64)
    chords=[(110.0,164.8,220.0),(123.5,185.0,246.9),(130.8,196.0,261.6),(110.0,174.6,220.0),(123.5,185.0,246.9),(130.8,196.0,261.6)]
    for idx in range(6):
        start,end=CUTS[idx],CUTS[idx+1]
        mask=(times>=start)&(times<end)
        t=times[mask]-start
        fade=np.minimum(1,np.minimum(t/.8, (end-start-t)/.85).clip(min=0))
        notes=chords[idx]
        wavepad=(np.sin(2*np.pi*notes[0]*t)*0.54+
                 np.sin(2*np.pi*notes[1]*t)*0.18+
                 np.sin(2*np.pi*notes[2]*t)*0.12)
        signal[mask] += 0.12*fade*wavepad
    for at in CUTS[:-1]:
        start=int(at*sr)
        length=int(.16*sr)
        n=np.arange(length,dtype=np.float64)/sr
        chirp=(np.sin(2*np.pi*(640*n+680*n*n))*np.exp(-24*n))*0.12
        signal[start:start+length] += chirp
    signal *= np.minimum(1,np.minimum(times/1.2, (DURATION-times)/1.2).clip(min=0))
    pcm=np.clip(signal*32767,-32767,32767).astype(np.int16)
    with wave.open(str(path),"wb") as wav:
        wav.setnchannels(1);wav.setsampwidth(2);wav.setframerate(sr)
        wav.writeframes(pcm.tobytes())

def make_srt():
    times=[("00:00:00,000","00:00:03,900","Your SKILL is now a TOOL.\n让 SKILL 真正成为工具。"),
           ("00:00:03,900","00:00:09,000","One readable document. Callable business logic.\n规则、数据、工具在同一份文档里。"),
           ("00:00:09,000","00:00:17,400","Five verified CLI calls, from quote to decision.\n真实 CLI 结果：询价、预算、模拟发单。"),
           ("00:00:17,400","00:00:23,400","¥120 spent: mock order. ¥360 spent: rejected.\n¥120 时模拟通过；¥360 时预算不足。"),
           ("00:00:23,400","00:00:29,000","Inspect the actual document in Marqdo View.\n查看 Marqdo 真实 View 页面。"),
           ("00:00:29,000","00:00:32,000","github.com/cflmy/marqdo\n开源 · Marqdo 2.7.1")]
    txt="\n\n".join(f"{i+1}\n{a} --> {b}\n{c}" for i,(a,b,c) in enumerate(times))+"\n"
    SUBTITLES.write_text(txt,encoding="utf-8")

def main():
    if not PROC.exists() or not VIEW.exists():
        raise FileNotFoundError("Expected procurement source and actual View screenshot")
    source=PROC.read_text("utf-8")
    assert "## 向采购商发工单" in source and "查询余额" in source
    assert VIEW_IMAGE.width >= 1200 and VIEW_IMAGE.height >= 700
    OUT.parent.mkdir(parents=True,exist_ok=True)
    POSTER_IMG=render(1.8)
    POSTER_IMG.save(POSTER,quality=92,optimize=True)
    make_srt()
    frames=int(DURATION*FPS)
    with tempfile.TemporaryDirectory() as tmp:
        unscored=Path(tmp)/"silent.mp4"
        music=Path(tmp)/"music.wav"
        cmd=["ffmpeg","-y","-hide_banner","-loglevel","error","-f","rawvideo",
             "-pixel_format","rgb24","-video_size",f"{W}x{H}","-framerate",str(FPS),
             "-i","pipe:0","-an","-c:v","libx264","-preset","veryfast","-crf","21",
             "-pix_fmt","yuv420p","-movflags","+faststart",str(unscored)]
        proc=subprocess.Popen(cmd,stdin=subprocess.PIPE,stderr=subprocess.PIPE)
        try:
            for k in range(frames):
                fr=render(k/FPS)
                proc.stdin.write(fr.tobytes())
                if k%120==0:
                    print("rendered",k,"/",frames,flush=True)
        finally:
            if proc.stdin:
                proc.stdin.close()
        stderr=proc.stderr.read().decode(errors="replace")
        rc=proc.wait(timeout=70)
        if rc:
            raise RuntimeError("Video encoder error: "+stderr)
        make_audio(music)
        subprocess.run(["ffmpeg","-y","-hide_banner","-loglevel","error",
                        "-i",str(unscored),"-i",str(music),"-c:v","copy","-c:a",
                        "aac","-b:a","128k","-shortest","-movflags","+faststart",
                        str(OUT)],check=True,timeout=90)
    print("VIDEO_WRITTEN",OUT,OUT.stat().st_size,flush=True)
    print("POSTER_WRITTEN",POSTER,POSTER.stat().st_size,flush=True)
    print("SUBTITLES_WRITTEN",SUBTITLES,SUBTITLES.stat().st_size,flush=True)
    ffprobe=subprocess.run(["ffprobe","-v","error","-show_entries",
                            "format=duration,size:stream=codec_name,width,height",
                            "-of","json",str(OUT)],capture_output=True,text=True,check=True)
    print("FFPROBE",ffprobe.stdout,flush=True)
    assert OUT.stat().st_size>1000000, "Suspiciously small encoded video"

if __name__=="__main__":
    main()
