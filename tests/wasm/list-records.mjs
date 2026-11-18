import test from "node:test";
import assert from "node:assert/strict";
import {renderListItems} from "../../crates/marqdo-wasm/js/marqdo-bridge.js";

test("typed list children retain text, controls and literal identities",()=>{
 const html=renderListItems({tag:"li",items:[{attrs:{"data-id":"row-1"},children:[
  {tag:"span",text:'A <tag> "quote" & literal'},
  {tag:"button",text:"删",attrs:{type:"button","data-id":'id" & value',disabled:false}}
 ]}]});
 assert.ok(html.includes("A &lt;tag&gt; &quot;quote&quot; &amp; literal"));
 assert.ok(html.includes('<button type="button" data-id="id&quot; &amp; value">删</button>'));
 assert.ok(!html.includes("<tag>") && !html.includes(" disabled"));
 assert.equal(renderListItems({items:["a",{text:"b"}]}),"<li>a</li><li>b</li>");
 assert.equal(renderListItems({items:[{html:"<em>explicit legacy</em>"}]}),"<li><em>explicit legacy</em></li>");
});
test("structural lists reject malformed nodes and bounded recursion",()=>{
 for(const item of [
  {tag:'button onclick="x"',text:"x"},{tag:"script",text:"x"},
  {attrs:{"data-id onload":"x"},text:"x"},{attrs:{onclick:"x"},text:"x"},
  {attrs:{href:"javascript:alert(1)"},text:"x"},
  {children:"not a list"},{text:"ambiguous",children:[]},{tag:"input",children:[{text:"x"}]}
 ]) assert.throws(()=>renderListItems({items:[item]}),/render_list/);
 let deep={text:"leaf"};
 for(let i=0;i<34;i++)deep={children:[deep]};
 assert.throws(()=>renderListItems({items:[deep]}),/limit/);
 assert.throws(()=>renderListItems({items:Array(10001).fill("x")}),/limit/);
});
