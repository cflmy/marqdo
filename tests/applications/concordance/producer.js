var bridge = data.map(row=>({id:row.id,tokens:row.text.normalize('NFC').replace(/[A-Z]/g,c=>c.toLowerCase()).split(/[ \n\t\r\u00a0]+/).filter(t=>t.length)}));
bridge.sort((a,b)=>Buffer.compare(Buffer.from(a.id),Buffer.from(b.id)));
bridge;
