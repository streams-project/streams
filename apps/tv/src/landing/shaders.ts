// GLSL for WebGL 1. Positions are px at 1920 by 1080 so the geometry stays
// the same at any render size.

var HEAD = "#ifdef GL_FRAGMENT_PRECISION_HIGH\nprecision highp float;\n#else\nprecision mediump float;\n#endif\n";

export var FULL_VERT =
  "attribute vec2 aPos;varying vec2 vP;void main(){vP=aPos;gl_Position=vec4(aPos.x*2.-1.,1.-aPos.y*2.,0.,1.);}";

// The opening scene in one pass: wordmark, mark, column, stripes, ring.
// uStripeRows holds the texture row of each band: mixed, violet, blue, sheen, thread.
export var OPENING_FRAG = HEAD + [
  "varying vec2 vP;",
  "uniform sampler2D uWord;uniform sampler2D uMark;uniform sampler2D uStripes;",
  "uniform vec3 uBlack;uniform vec3 uRingColour;",
  "uniform vec4 uWordBox;uniform vec2 uWordState;",
  "uniform vec4 uMarkBox;uniform vec3 uMarkState;",
  "uniform vec4 uStripeState;uniform vec2 uZoom;",
  "uniform vec4 uRing;uniform vec3 uRingState;",
  "uniform vec4 uRows;uniform float uThreadRow;",
  "void main(){",
  "float x=vP.x-.5;",
  "vec3 mixedC=texture2D(uStripes,vec2(x/uZoom.x+.5,uRows.x)).rgb;",
  "vec3 warmC=texture2D(uStripes,vec2(x/uZoom.x+.5,uRows.y)).rgb;",
  "vec3 coolC=texture2D(uStripes,vec2(x/uZoom.y+.5,uRows.z)).rgb;",
  "float sheen=texture2D(uStripes,vec2(x/(uZoom.x*1.7)+.5,uRows.w)).r;",
  "vec3 st=mix(mixedC,warmC+coolC,uStripeState.y)*(.7+.4*sheen);",
  "vec3 col=uBlack;",
  // Wordmark: clipped on the right, letters giving way from the bottom and
  // leaving a fragment at the top.
  "if(uWordState.x>0.){",
  "vec2 q=vec2((vP.x-uWordBox.x)/(uWordBox.y-uWordBox.x),(vP.y-uWordBox.z)/(uWordBox.w-uWordBox.z));",
  "if(q.x>0.&&q.x<1.&&q.y>0.&&q.y<1.){",
  "vec4 l=texture2D(uWord,q);",
  "float n=texture2D(uStripes,vec2(q.x*2.7,uThreadRow)).r;",
  "float edge=(uWordState.y-vP.x)/.035;",
  "float a=l.a*uWordState.x*clamp(edge*1.2+(1.-q.y)*.55+n*.3-.35,0.,1.);",
  "vec3 lc=l.a>.001?l.rgb/l.a:vec3(0.);",
  "col=mix(col,lc,a);",
  "}}",
  // Mark: cut apart by threads as it folds, taking the stripe colours with it.
  "if(uMarkState.x>0.){",
  "vec2 m=vec2((vP.x-uMarkBox.x)/(uMarkBox.y-uMarkBox.x),(vP.y-uMarkBox.z)/(uMarkBox.w-uMarkBox.z));",
  "if(m.x>0.&&m.x<1.&&m.y>0.&&m.y<1.){",
  "vec4 l=texture2D(uMark,m);",
  "float n=texture2D(uStripes,vec2(m.x*.42+.3,uThreadRow)).r;",
  "float f=uMarkState.y*(.1+.75*m.x);",
  "float a=l.a*smoothstep(f-.04,f+.04,n+.02)*uMarkState.x;",
  "vec3 lc=l.a>.001?l.rgb/l.a:vec3(0.);",
  "lc=mix(lc,st*1.2,uMarkState.z);",
  "col=mix(col,lc,a);",
  "}}",
  // Column: the stripes, held to a width that opens to the whole picture, and
  // to the mark's own height until the fold hands the whole picture over.
  "if(uStripeState.z>0.){",
  "float d=abs(x)-uStripeState.x;",
  "float tall=smoothstep(uMarkBox.z-.004,uMarkBox.z+.004,vP.y)*(1.-smoothstep(uMarkBox.w-.004,uMarkBox.w+.004,vP.y));",
  "float k=(1.-smoothstep(-.0015,.0015,d))*uStripeState.z*tall;",
  "col=mix(col,st,k);",
  "col+=st*.28*exp(-max(d,0.)*34.)*(1.-k)*uStripeState.z*tall;",
  "}",
  "col=mix(uBlack,col,uStripeState.w);",
  "if(uRingState.z>0.){",
  "vec2 rp=vP*vec2(1920.,1080.)-uRing.xy;",
  "float r=length(rp);",
  "float dr=abs(r-uRing.z)-uRing.w*.5;",
  "if(dr<1.5){",
  "float ang=atan(rp.y,rp.x)/6.2831853+.5;",
  "ang=fract(ang-uRingState.x);",
  "float k=clamp(.5-dr,0.,1.)*smoothstep(0.,.008,ang)*(1.-smoothstep(uRingState.y-.008,uRingState.y,ang))*uRingState.z;",
  "col=mix(col,uRingColour,k);",
  "}}",
  "gl_FragColor=vec4(col,1.);",
  "}"
].join("");

export var TILE_VERT = [
  "attribute vec3 aPos;attribute vec2 aUv;attribute vec2 aLocal;attribute vec2 aHalf;attribute vec3 aMisc;",
  "uniform vec3 uPivot;uniform float uScatter;uniform vec4 uCard;uniform float uUnderText;",
  "varying vec2 vUv;varying vec2 vLocal;varying vec2 vHalf;varying vec2 vBright;",
  "void main(){",
  "vec2 p=aPos.xy+aMisc.yz*uScatter;",
  "p=uPivot.xy+(p-uPivot.xy)*uPivot.z;",
  "vec2 c=(p-uCard.xy)/uCard.zw;",
  "gl_Position=vec4(vec2(c.x*2.-1.,1.-c.y*2.)*aPos.z,0.,aPos.z);",
  "vUv=aUv;vLocal=aLocal;vHalf=aHalf;vBright=vec2(abs(aMisc.x),aMisc.x<0.?uUnderText:1.);",
  "}"
].join("");

export var TILE_FRAG = HEAD + [
  "varying vec2 vUv;varying vec2 vLocal;varying vec2 vHalf;varying vec2 vBright;",
  "uniform sampler2D uAtlas;uniform float uAlpha;uniform float uRadius;uniform float uAa;",
  "void main(){",
  "vec2 q=abs(vLocal)-vHalf+uRadius;",
  "float d=min(max(q.x,q.y),0.)+length(max(q,0.))-uRadius;",
  "float a=clamp(.5-d*uAa,0.,1.)*uAlpha*vBright.y;",
  "gl_FragColor=vec4(texture2D(uAtlas,vUv).rgb*vBright.x*a,a);",
  "}"
].join("");

export var TEXT_VERT = [
  "attribute vec2 aPos;attribute vec2 aUv;",
  "uniform vec3 uPivot;uniform vec4 uCard;varying vec2 vUv;",
  "void main(){",
  "vec2 p=uPivot.xy+(aPos-uPivot.xy)*uPivot.z;",
  "vec2 c=(p-uCard.xy)/uCard.zw;",
  "gl_Position=vec4(c.x*2.-1.,1.-c.y*2.,0.,1.);",
  "vUv=aUv;",
  "}"
].join("");

export var TEXT_FRAG = HEAD + [
  "varying vec2 vUv;uniform sampler2D uTex;uniform vec4 uTint;",
  "void main(){vec4 t=texture2D(uTex,vUv);gl_FragColor=vec4(t.rgb*uTint.rgb,t.a)*uTint.a;}"
].join("");

export var BLUR_FRAG = HEAD + [
  "varying vec2 vP;uniform sampler2D uTex;uniform vec2 uDir;uniform vec3 uOff;uniform vec4 uWeight;",
  "void main(){",
  "vec2 t=vec2(vP.x,1.-vP.y);",
  "vec4 c=texture2D(uTex,t)*uWeight.x;",
  "c+=(texture2D(uTex,t+uDir*uOff.x)+texture2D(uTex,t-uDir*uOff.x))*uWeight.y;",
  "c+=(texture2D(uTex,t+uDir*uOff.y)+texture2D(uTex,t-uDir*uOff.y))*uWeight.z;",
  "c+=(texture2D(uTex,t+uDir*uOff.z)+texture2D(uTex,t-uDir*uOff.z))*uWeight.w;",
  "gl_FragColor=c;",
  "}"
].join("");

// One full screen pass: backdrop, band, card, layer, glass edge.
// Glass edge: a 3 px core with a 3.5 px outer glow, the measured cross profile.
export var CARD_FRAG = HEAD + [
  "varying vec2 vP;",
  "uniform sampler2D uLayer;",
  "uniform vec3 uBlack;uniform vec3 uBandA;uniform vec3 uBandB;uniform vec3 uEdge;",
  "uniform vec3 uFillTop;uniform vec3 uFillMid;uniform vec3 uFillLow;uniform vec3 uDarkTop;uniform vec3 uDarkLow;",
  "uniform vec4 uCard;uniform vec4 uState;uniform float uPx;uniform float uRadius;",
  "void main(){",
  "vec2 p=vP*vec2(1920.,1080.);",
  "float band=uState.x*(1.-smoothstep(.1,.95,vP.x))*(1.-vP.y*.55);",
  "vec3 col=uBlack+mix(uBandA,uBandB,smoothstep(.0,.55,vP.x))*band;",
  "vec2 c=uCard.xy+uCard.zw*.5;",
  "vec2 q=abs(p-c)-uCard.zw*.5+uRadius;",
  "float d=min(max(q.x,q.y),0.)+length(max(q,0.))-uRadius;",
  "if(d<8.){",
  "vec2 l=(p-uCard.xy)/uCard.zw;",
  "vec3 dark=mix(uDarkTop,uDarkLow,l.y);",
  "vec3 fill=mix(uFillTop,uFillMid,smoothstep(.05,.55,l.y));",
  "fill=mix(fill,uFillLow,smoothstep(.70,.75,l.y));",
  "vec3 inner=mix(dark,fill,uState.y);",
  "vec4 t=texture2D(uLayer,vec2(l.x,1.-l.y));",
  "inner=inner*(1.-t.a*uState.z)+t.rgb*uState.z*uState.w;",
  "inner*=1.-.62*smoothstep(.70,.93,l.y);",
  "float px=uPx;",
  "col=mix(col,inner,clamp(.5-d/px,0.,1.));",
  "float core=clamp((d+3.)/px+.5,0.,1.)*clamp(.5-d/px,0.,1.);",
  "float glow=d>0.?.35*exp(-d/3.5):0.;",
  "col=mix(col,uEdge,core);",
  "col+=uEdge*glow;",
  "}",
  "gl_FragColor=vec4(col,1.);",
  "}"
].join("");

export var RECT_VERT = [
  "attribute vec2 aPos;uniform vec4 uRect;uniform vec4 uUv;varying vec2 vUv;varying vec2 vL;",
  "void main(){",
  "vec2 p=uRect.xy+aPos*uRect.zw;",
  "gl_Position=vec4(p.x/960.-1.,1.-p.y/540.,0.,1.);",
  "vUv=uUv.xy+aPos*uUv.zw;vL=aPos;",
  "}"
].join("");

export var RECT_FRAG = HEAD + [
  "varying vec2 vUv;uniform sampler2D uTex;uniform float uAlpha;",
  "void main(){gl_FragColor=texture2D(uTex,vUv)*uAlpha;}"
].join("");

// Next card: rounded on the left, runs off the right edge, art falls to dark at the foot.
export var PEEK_FRAG = HEAD + [
  "varying vec2 vUv;varying vec2 vL;uniform sampler2D uTex;uniform float uAlpha;uniform vec2 uSize;uniform float uRadius;uniform float uPx;",
  "void main(){",
  "vec2 p=vL*uSize;",
  "vec2 q=vec2(uRadius-p.x,max(uRadius-p.y,p.y-uSize.y+uRadius));",
  "float d=length(max(q,0.))-uRadius;",
  "float a=clamp(.5-d/uPx,0.,1.)*uAlpha;",
  "vec3 c=texture2D(uTex,vUv).rgb*(1.-.9*smoothstep(.62,.98,vL.y))*.86;",
  "gl_FragColor=vec4(c*a,a);",
  "}"
].join("");
