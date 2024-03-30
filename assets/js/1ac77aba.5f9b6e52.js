"use strict";(self.webpackChunkmy_website=self.webpackChunkmy_website||[]).push([[303],{3905:(e,r,a)=>{a.d(r,{Zo:()=>o,kt:()=>m});var t=a(7294);function n(e,r,a){return r in e?Object.defineProperty(e,r,{value:a,enumerable:!0,configurable:!0,writable:!0}):e[r]=a,e}function i(e,r){var a=Object.keys(e);if(Object.getOwnPropertySymbols){var t=Object.getOwnPropertySymbols(e);r&&(t=t.filter((function(r){return Object.getOwnPropertyDescriptor(e,r).enumerable}))),a.push.apply(a,t)}return a}function l(e){for(var r=1;r<arguments.length;r++){var a=null!=arguments[r]?arguments[r]:{};r%2?i(Object(a),!0).forEach((function(r){n(e,r,a[r])})):Object.getOwnPropertyDescriptors?Object.defineProperties(e,Object.getOwnPropertyDescriptors(a)):i(Object(a)).forEach((function(r){Object.defineProperty(e,r,Object.getOwnPropertyDescriptor(a,r))}))}return e}function s(e,r){if(null==e)return{};var a,t,n=function(e,r){if(null==e)return{};var a,t,n={},i=Object.keys(e);for(t=0;t<i.length;t++)a=i[t],r.indexOf(a)>=0||(n[a]=e[a]);return n}(e,r);if(Object.getOwnPropertySymbols){var i=Object.getOwnPropertySymbols(e);for(t=0;t<i.length;t++)a=i[t],r.indexOf(a)>=0||Object.prototype.propertyIsEnumerable.call(e,a)&&(n[a]=e[a])}return n}var u=t.createContext({}),c=function(e){var r=t.useContext(u),a=r;return e&&(a="function"==typeof e?e(r):l(l({},r),e)),a},o=function(e){var r=c(e.components);return t.createElement(u.Provider,{value:r},e.children)},d={inlineCode:"code",wrapper:function(e){var r=e.children;return t.createElement(t.Fragment,{},r)}},p=t.forwardRef((function(e,r){var a=e.components,n=e.mdxType,i=e.originalType,u=e.parentName,o=s(e,["components","mdxType","originalType","parentName"]),p=c(a),m=n,k=p["".concat(u,".").concat(m)]||p[m]||d[m]||i;return a?t.createElement(k,l(l({ref:r},o),{},{components:a})):t.createElement(k,l({ref:r},o))}));function m(e,r){var a=arguments,n=r&&r.mdxType;if("string"==typeof e||n){var i=a.length,l=new Array(i);l[0]=p;var s={};for(var u in r)hasOwnProperty.call(r,u)&&(s[u]=r[u]);s.originalType=e,s.mdxType="string"==typeof e?e:n,l[1]=s;for(var c=2;c<i;c++)l[c]=a[c];return t.createElement.apply(null,l)}return t.createElement.apply(null,a)}p.displayName="MDXCreateElement"},8544:(e,r,a)=>{a.r(r),a.d(r,{assets:()=>u,contentTitle:()=>l,default:()=>d,frontMatter:()=>i,metadata:()=>s,toc:()=>c});var t=a(7462),n=(a(7294),a(3905));const i={},l="MultieraSundaeSwapV1MeanPriceTask",s={unversionedId:"indexer/Tasks/MultieraSundaeSwapV1MeanPriceTask",id:"indexer/Tasks/MultieraSundaeSwapV1MeanPriceTask",title:"MultieraSundaeSwapV1MeanPriceTask",description:"Adds SundaeSwap V1 mean price updates to the database",source:"@site/docs/indexer/Tasks/MultieraSundaeSwapV1MeanPriceTask.md",sourceDirName:"indexer/Tasks",slug:"/indexer/Tasks/MultieraSundaeSwapV1MeanPriceTask",permalink:"/carp/docs/indexer/Tasks/MultieraSundaeSwapV1MeanPriceTask",draft:!1,editUrl:"https://github.com/dcSpark/carp/docs/indexer/Tasks/MultieraSundaeSwapV1MeanPriceTask.md",tags:[],version:"current",frontMatter:{},sidebar:"tutorialSidebar",previous:{title:"MultieraStakeCredentialTask",permalink:"/carp/docs/indexer/Tasks/MultieraStakeCredentialTask"},next:{title:"MultieraSundaeSwapV1SwapTask",permalink:"/carp/docs/indexer/Tasks/MultieraSundaeSwapV1SwapTask"}},u={},c=[{value:"Era",id:"era",level:2},{value:"Dependencies",id:"dependencies",level:2},{value:"Data accessed",id:"data-accessed",level:2},{value:"Reads from",id:"reads-from",level:4},{value:"Full source",id:"full-source",level:2}],o={toc:c};function d(e){let{components:r,...a}=e;return(0,n.kt)("wrapper",(0,t.Z)({},o,a,{components:r,mdxType:"MDXLayout"}),(0,n.kt)("h1",{id:"multierasundaeswapv1meanpricetask"},"MultieraSundaeSwapV1MeanPriceTask"),(0,n.kt)("p",null,"Adds SundaeSwap V1 mean price updates to the database"),(0,n.kt)("details",null,(0,n.kt)("summary",null,"Configuration"),(0,n.kt)("pre",null,(0,n.kt)("code",{parentName:"pre",className:"language-rust"},"#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]\npub struct EmptyConfig {}\n\n"))),(0,n.kt)("h2",{id:"era"},"Era"),(0,n.kt)("p",null,(0,n.kt)("inlineCode",{parentName:"p"},"multiera")),(0,n.kt)("h2",{id:"dependencies"},"Dependencies"),(0,n.kt)("ul",null,(0,n.kt)("li",{parentName:"ul"},(0,n.kt)("a",{parentName:"li",href:"./MultieraAddressTask"},"MultieraAddressTask"))),(0,n.kt)("h2",{id:"data-accessed"},"Data accessed"),(0,n.kt)("h4",{id:"reads-from"},"Reads from"),(0,n.kt)("ul",null,(0,n.kt)("li",{parentName:"ul"},(0,n.kt)("inlineCode",{parentName:"li"},"multiera_txs")),(0,n.kt)("li",{parentName:"ul"},(0,n.kt)("inlineCode",{parentName:"li"},"multiera_addresses"))),(0,n.kt)("h2",{id:"full-source"},"Full source"),(0,n.kt)("p",null,(0,n.kt)("a",{parentName:"p",href:"https://github.com/dcSpark/carp/tree/main/indexer/indexer/tasks/src/multiera/multiera_sundaeswap_v1_mean_price.rs"},"source")))}d.isMDXComponent=!0}}]);