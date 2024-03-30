"use strict";(self.webpackChunkmy_website=self.webpackChunkmy_website||[]).push([[7028],{3905:(e,t,r)=>{r.d(t,{Zo:()=>p,kt:()=>m});var n=r(7294);function a(e,t,r){return t in e?Object.defineProperty(e,t,{value:r,enumerable:!0,configurable:!0,writable:!0}):e[t]=r,e}function i(e,t){var r=Object.keys(e);if(Object.getOwnPropertySymbols){var n=Object.getOwnPropertySymbols(e);t&&(n=n.filter((function(t){return Object.getOwnPropertyDescriptor(e,t).enumerable}))),r.push.apply(r,n)}return r}function l(e){for(var t=1;t<arguments.length;t++){var r=null!=arguments[t]?arguments[t]:{};t%2?i(Object(r),!0).forEach((function(t){a(e,t,r[t])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(r)):i(Object(r)).forEach((function(t){Object.defineProperty(e,t,Object.getOwnPropertyDescriptor(r,t))}))}return e}function s(e,t){if(null==e)return{};var r,n,a=function(e,t){if(null==e)return{};var r,n,a={},i=Object.keys(e);for(n=0;n<i.length;n++)r=i[n],t.indexOf(r)>=0||(a[r]=e[r]);return a}(e,t);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(n=0;n<i.length;n++)r=i[n],t.indexOf(r)>=0||Object.prototype.propertyIsEnumerable.call(e,r)&&(a[r]=e[r])}return a}var u=n.createContext({}),o=function(e){var t=n.useContext(u),r=t;return e&&(r="function"==typeof e?e(t):l(l({},t),e)),r},p=function(e){var t=o(e.components);return n.createElement(u.Provider,{value:t},e.children)},d={inlineCode:"code",wrapper:function(e){var t=e.children;return n.createElement(n.Fragment,{},t)}},c=n.forwardRef((function(e,t){var r=e.components,a=e.mdxType,i=e.originalType,u=e.parentName,p=s(e,["components","mdxType","originalType","parentName"]),c=o(r),m=a,k=c["".concat(u,".").concat(m)]||c[m]||d[m]||i;return r?n.createElement(k,l(l({ref:t},p),{},{components:r})):n.createElement(k,l({ref:t},p))}));function m(e,t){var r=arguments,a=t&&t.mdxType;if("string"==typeof e||a){var i=r.length,l=new Array(i);l[0]=c;var s={};for(var u in t)hasOwnProperty.call(t,u)&&(s[u]=t[u]);s.originalType=e,s.mdxType="string"==typeof e?e:a,l[1]=s;for(var o=2;o<i;o++)l[o]=r[o];return n.createElement.apply(null,l)}return n.createElement.apply(null,r)}c.displayName="MDXCreateElement"},4266:(e,t,r)=>{r.r(t),r.d(t,{assets:()=>u,contentTitle:()=>l,default:()=>d,frontMatter:()=>i,metadata:()=>s,toc:()=>o});var n=r(7462),a=(r(7294),r(3905));const i={},l="MultieraUsedInputTask",s={unversionedId:"indexer/Tasks/MultieraUsedInputTask",id:"indexer/Tasks/MultieraUsedInputTask",title:"MultieraUsedInputTask",description:"Adds the used inputs to the database \\(regular inputs in most cases, collateral inputs if tx fails\\)",source:"@site/docs/indexer/Tasks/MultieraUsedInputTask.md",sourceDirName:"indexer/Tasks",slug:"/indexer/Tasks/MultieraUsedInputTask",permalink:"/carp/docs/indexer/Tasks/MultieraUsedInputTask",draft:!1,editUrl:"https://github.com/dcSpark/carp/docs/indexer/Tasks/MultieraUsedInputTask.md",tags:[],version:"current",frontMatter:{},sidebar:"tutorialSidebar",previous:{title:"MultieraUnusedInputTask",permalink:"/carp/docs/indexer/Tasks/MultieraUnusedInputTask"},next:{title:"MultieraWingRidersV1MeanPriceTask",permalink:"/carp/docs/indexer/Tasks/MultieraWingRidersV1MeanPriceTask"}},u={},o=[{value:"Era",id:"era",level:2},{value:"Dependencies",id:"dependencies",level:2},{value:"Data accessed",id:"data-accessed",level:2},{value:"Reads from",id:"reads-from",level:4},{value:"Writes to",id:"writes-to",level:4},{value:"Full source",id:"full-source",level:2}],p={toc:o};function d(e){let{components:t,...r}=e;return(0,a.kt)("wrapper",(0,n.Z)({},p,r,{components:t,mdxType:"MDXLayout"}),(0,a.kt)("h1",{id:"multierausedinputtask"},"MultieraUsedInputTask"),(0,a.kt)("p",null,"Adds the used inputs to the database ","(","regular inputs in most cases, collateral inputs if tx fails",")"),(0,a.kt)("details",null,(0,a.kt)("summary",null,"Configuration"),(0,a.kt)("pre",null,(0,a.kt)("code",{parentName:"pre",className:"language-rust"},"#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]\npub struct ReadonlyConfig {\n    pub readonly: bool,\n}\n\n"))),(0,a.kt)("h2",{id:"era"},"Era"),(0,a.kt)("p",null,(0,a.kt)("inlineCode",{parentName:"p"},"multiera")),(0,a.kt)("h2",{id:"dependencies"},"Dependencies"),(0,a.kt)("ul",null,(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("a",{parentName:"li",href:"./MultieraOutputTask"},"MultieraOutputTask"))),(0,a.kt)("h2",{id:"data-accessed"},"Data accessed"),(0,a.kt)("h4",{id:"reads-from"},"Reads from"),(0,a.kt)("ul",null,(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("inlineCode",{parentName:"li"},"multiera_txs"))),(0,a.kt)("h4",{id:"writes-to"},"Writes to"),(0,a.kt)("ul",null,(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("inlineCode",{parentName:"li"},"vkey_relation_map")),(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("inlineCode",{parentName:"li"},"multiera_used_inputs")),(0,a.kt)("li",{parentName:"ul"},(0,a.kt)("inlineCode",{parentName:"li"},"multiera_used_inputs_to_outputs_map"))),(0,a.kt)("h2",{id:"full-source"},"Full source"),(0,a.kt)("p",null,(0,a.kt)("a",{parentName:"p",href:"https://github.com/dcSpark/carp/tree/main/indexer/indexer/tasks/src/multiera/multiera_used_inputs.rs"},"source")))}d.isMDXComponent=!0}}]);