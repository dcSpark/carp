"use strict";(self.webpackChunkmy_website=self.webpackChunkmy_website||[]).push([[6650],{3905:(e,t,r)=>{r.d(t,{Zo:()=>c,kt:()=>k});var a=r(7294);function n(e,t,r){return t in e?Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}):e[t]=r,e}function i(e,t){var r=Object.keys(e);if(Object.getOwnPropertySymbols){var a=Object.getOwnPropertySymbols(e);t&&(a=a.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),r.push.apply(r,a)}return r}function l(e){for(var t=1;t<arguments.length;t++){var r=null!=arguments[t]?arguments[t]:{};t%2?i(Object(r),!0).forEach((function(t){n(e,t,r[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(r)):i(Object(r)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(r,t))}))}return e}function o(e,t){if(null==e)return{};var r,a,n=function(e,t){if(null==e)return{};var r,a,n={},i=Object.keys(e);for(a=0;a<i.length;a++)r=i[a],t.indexOf(r)>=0||(n[r]=e[r]);return n}(e,t);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(a=0;a<i.length;a++)r=i[a],t.indexOf(r)>=0||Object.prototype.propertyIsEnumerable.call(e,r)&&(n[r]=e[r])}return n}var s=a.createContext({}),u=function(e){var t=a.useContext(s),r=t;return e&&(r="function"==typeof e?e(t):l(l({},t),e)),r},c=function(e){var t=u(e.components);return a.createElement(s.Provider,{value:t},e.children)},d={inlineCode:"code",wrapper:function(e){var t=e.children;return a.createElement(a.Fragment,{},t)}},p=a.forwardRef((function(e,t){var r=e.components,n=e.mdxType,i=e.originalType,s=e.parentName,c=o(e,["components","mdxType","originalType","parentName"]),p=u(r),k=n,m=p["".concat(s,".").concat(k)]||p[k]||d[k]||i;return r?a.createElement(m,l(l({ref:t},c),{},{components:r})):a.createElement(m,l({ref:t},c))}));function k(e,t){var r=arguments,n=t&&t.mdxType;if("string"==typeof e||n){var i=r.length,l=new Array(i);l[0]=p;var o={};for(var s in t)hasOwnProperty.call(t,s)&&(o[s]=t[s]);o.originalType=e,o.mdxType="string"==typeof e?e:n,l[1]=o;for(var u=2;u<i;u++)l[u]=r[u];return a.createElement.apply(null,l)}return a.createElement.apply(null,r)}p.displayName="MDXCreateElement"},8086:(e,t,r)=>{r.r(t),r.d(t,{assets:()=>s,contentTitle:()=>l,default:()=>d,frontMatter:()=>i,metadata:()=>o,toc:()=>u});var a=r(7462),n=(r(7294),r(3905));const i={},l="MultieraTransactionTask",o={unversionedId:"indexer/Tasks/MultieraTransactionTask",id:"indexer/Tasks/MultieraTransactionTask",title:"MultieraTransactionTask",description:"Adds the transactions in the block to the database",source:"@site/docs/indexer/Tasks/MultieraTransactionTask.md",sourceDirName:"indexer/Tasks",slug:"/indexer/Tasks/MultieraTransactionTask",permalink:"/carp/docs/indexer/Tasks/MultieraTransactionTask",draft:!1,editUrl:"https://github.com/dcSpark/carp/docs/indexer/Tasks/MultieraTransactionTask.md",tags:[],version:"current",frontMatter:{},sidebar:"tutorialSidebar",previous:{title:"MultieraSundaeSwapV1SwapTask",permalink:"/carp/docs/indexer/Tasks/MultieraSundaeSwapV1SwapTask"},next:{title:"MultieraTxCredentialRelationTask",permalink:"/carp/docs/indexer/Tasks/MultieraTxCredentialRelationTask"}},s={},u=[{value:"Era",id:"era",level:2},{value:"Dependencies",id:"dependencies",level:2},{value:"Data accessed",id:"data-accessed",level:2},{value:"Reads from",id:"reads-from",level:4},{value:"Writes to",id:"writes-to",level:4},{value:"Full source",id:"full-source",level:2}],c={toc:u};function d(e){let{components:t,...r}=e;return(0,n.kt)("wrapper",(0,a.Z)({},c,r,{components:t,mdxType:"MDXLayout"}),(0,n.kt)("h1",{id:"multieratransactiontask"},"MultieraTransactionTask"),(0,n.kt)("p",null,"Adds the transactions in the block to the database"),(0,n.kt)("details",null,(0,n.kt)("summary",null,"Configuration"),(0,n.kt)("pre",null,(0,n.kt)("code",{parentName:"pre",className:"language-rust"},"use super::PayloadConfig::PayloadConfig;\nuse super::ReadonlyConfig::ReadonlyConfig;\n\n#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]\npub struct PayloadAndReadonlyConfig {\n    pub include_payload: bool,\n    pub readonly: bool,\n}\n\n"))),(0,n.kt)("h2",{id:"era"},"Era"),(0,n.kt)("p",null,(0,n.kt)("inlineCode",{parentName:"p"},"multiera")),(0,n.kt)("h2",{id:"dependencies"},"Dependencies"),(0,n.kt)("ul",null,(0,n.kt)("li",{parentName:"ul"},(0,n.kt)("a",{parentName:"li",href:"./MultieraBlockTask"},"MultieraBlockTask"))),(0,n.kt)("h2",{id:"data-accessed"},"Data accessed"),(0,n.kt)("h4",{id:"reads-from"},"Reads from"),(0,n.kt)("ul",null,(0,n.kt)("li",{parentName:"ul"},(0,n.kt)("inlineCode",{parentName:"li"},"multiera_block"))),(0,n.kt)("h4",{id:"writes-to"},"Writes to"),(0,n.kt)("ul",null,(0,n.kt)("li",{parentName:"ul"},(0,n.kt)("inlineCode",{parentName:"li"},"multiera_txs"))),(0,n.kt)("h2",{id:"full-source"},"Full source"),(0,n.kt)("p",null,(0,n.kt)("a",{parentName:"p",href:"https://github.com/dcSpark/carp/tree/main/indexer/indexer/tasks/src/multiera/multiera_txs.rs"},"source")))}d.isMDXComponent=!0}}]);