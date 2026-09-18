/* Raw OpenGL drawing in a CSS canvas, linked only to the Native UI SDK.
   Build: cc triangle.c -I/path/to/sdk -L/path/to/sdk -lnative_ui -o triangle
   Place the SDK library beside the executable and configure its runtime path.
   The normal language example builder's --backend opengl demonstrates the same
   host with portable drawing helpers; this file shows the actual shader calls. */
#include "native_ui.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#define GL_CALL __stdcall
#else
#define GL_CALL
#endif
#define GL_FN(result, name, args) typedef result (GL_CALL *name##Fn) args; static name##Fn name
GL_FN(uint32_t, glCreateShader, (uint32_t));
GL_FN(void, glShaderSource, (uint32_t,int,const char *const *,const int *));
GL_FN(void, glCompileShader, (uint32_t));
GL_FN(void, glGetShaderiv, (uint32_t,uint32_t,int *));
GL_FN(void, glGetShaderInfoLog, (uint32_t,int,int *,char *));
GL_FN(uint32_t, glCreateProgram, (void));
GL_FN(void, glAttachShader, (uint32_t,uint32_t));
GL_FN(void, glDeleteShader, (uint32_t));
GL_FN(void, glLinkProgram, (uint32_t));
GL_FN(void, glGetProgramiv, (uint32_t,uint32_t,int *));
GL_FN(void, glGetProgramInfoLog, (uint32_t,int,int *,char *));
GL_FN(void, glUseProgram, (uint32_t));
GL_FN(void, glDeleteProgram, (uint32_t));
GL_FN(void, glGenVertexArrays, (int,uint32_t *));
GL_FN(void, glBindVertexArray, (uint32_t));
GL_FN(void, glDeleteVertexArrays, (int,const uint32_t *));
GL_FN(int, glGetUniformLocation, (uint32_t,const char *));
GL_FN(void, glUniform1f, (int,float));
GL_FN(void, glDrawArrays, (uint32_t,int,int));
#define LOAD(name) do { const void *p=nui_window_gl_proc_address(window,#name); check(p!=NULL); memcpy(&name,&p,sizeof name); } while(0)
static uint32_t program,vao;
static int aspect_location;
static void check(int yes){if(!yes){fprintf(stderr,"%s\n",nui_last_error());exit(1);}}
static uint32_t shader(uint32_t kind,const char *source){
    uint32_t s=glCreateShader(kind);glShaderSource(s,1,&source,NULL);glCompileShader(s);
    int ok=0;glGetShaderiv(s,0x8B81,&ok);
    if(!ok){char log[2048];glGetShaderInfoLog(s,sizeof log,NULL,log);fprintf(stderr,"%s\n",log);exit(1);}return s;
}
static int draw(void *data,void *painter,NuiString id,NuiRect bounds,NuiRect content,NuiRect clip){
    (void)data;(void)id;(void)bounds;(void)content;(void)clip;
    uint32_t width,height;
    if(nui_draw_canvas_pixel_size(painter,&width,&height)!=1)return 1;
    /* Viewport and clip are already applied by Native UI. No window coordinates. */
    glUseProgram(program);glBindVertexArray(vao);
    glUniform1f(aspect_location,(float)width/(float)height);
    glDrawArrays(0x0004,0,3); /* GL_TRIANGLES */
    return 0;
}
/* Read the canonical skin from the repository or the SDK's bundled themes. */
static char *styles(const char *theme,const char *executable){
    const char *file=NULL;
    if(strcmp(theme,"win11")==0)file="windows-11-fluent.css";
    if(strcmp(theme,"macos")==0)file="macos-light.css";
    if(strcmp(theme,"xp")==0)file="windows-xp-luna.css";
    if(!file){fprintf(stderr,"Theme must be xp, macos or win11\n");exit(1);}
    char path[4096];FILE *f=NULL;
    const char *slash=strrchr(executable,'/');
#ifdef _WIN32
    const char *backslash=strrchr(executable,'\\');
    if(backslash && (!slash || backslash>slash))slash=backslash;
#endif
    if(slash){snprintf(path,sizeof path,"%.*s/themes/%s",(int)(slash-executable),executable,file);f=fopen(path,"rb");}
    if(!f){snprintf(path,sizeof path,"themes/%s",file);f=fopen(path,"rb");}
    if(!f){snprintf(path,sizeof path,"examples/showcase/ui/themes/%s",file);f=fopen(path,"rb");}
    if(!f){fprintf(stderr,"Cannot read theme %s; run from the repository or keep SDK themes beside the executable.\n",file);exit(1);}
    const char *layout="\ncanvas{position:absolute;left:0px;top:0px;width:100%;height:100%;padding:24px;background:#132036}button{position:absolute;left:24px;top:24px;width:120px}";
    check(fseek(f,0,SEEK_END)==0);long size=ftell(f);check(size>=0);rewind(f);
    char *css=(char *)malloc((size_t)size+strlen(layout)+1);check(css!=NULL);
    check(fread(css,1,(size_t)size,f)==(size_t)size);fclose(f);strcpy(css+size,layout);return css;
}
int main(int argc,char **argv){
    /* Defaults to macOS; pass --theme win11 or --theme xp for another skin. */
    int smoke=0;const char *theme="macos";
    for(int i=1;i<argc;i++){
        if(strcmp(argv[i],"--smoke-test")==0)smoke=1;
        else if(strcmp(argv[i],"--theme")==0 && i+1<argc)theme=argv[++i];
        else {fprintf(stderr,"Usage: triangle [--theme xp|macos|win11] [--smoke-test]\n");return 1;}
    }
    char *css=styles(theme,argv[0]);
    NuiWindow *window=nui_window_open("OpenGL canvas from C",800,520,smoke,NUI_BACKEND_OPENGL);check(window!=NULL);
    LOAD(glCreateShader);LOAD(glShaderSource);LOAD(glCompileShader);LOAD(glGetShaderiv);LOAD(glGetShaderInfoLog);
    LOAD(glCreateProgram);LOAD(glAttachShader);LOAD(glDeleteShader);LOAD(glLinkProgram);LOAD(glGetProgramiv);LOAD(glGetProgramInfoLog);
    LOAD(glUseProgram);LOAD(glDeleteProgram);LOAD(glGenVertexArrays);LOAD(glBindVertexArray);LOAD(glDeleteVertexArrays);
    LOAD(glGetUniformLocation);LOAD(glUniform1f);LOAD(glDrawArrays);
    uint32_t vs=shader(0x8B31,"#version 330 core\nuniform float aspect;out vec3 color;void main(){vec2 p[3]=vec2[3](vec2(0.,.7),vec2(-.6,-.4),vec2(.6,-.4));vec3 c[3]=vec3[3](vec3(.3,.6,1.),vec3(.9,.3,.5),vec3(.2,.9,.6));vec2 v=p[gl_VertexID];v.x/=aspect;gl_Position=vec4(v,0.,1.);color=c[gl_VertexID];}");
    uint32_t fs=shader(0x8B30,"#version 330 core\nin vec3 color;out vec4 pixel;void main(){pixel=vec4(color,1.);}");
    program=glCreateProgram();glAttachShader(program,vs);glAttachShader(program,fs);glDeleteShader(vs);glDeleteShader(fs);glLinkProgram(program);
    int linked=0;glGetProgramiv(program,0x8B82,&linked);if(!linked){char log[2048];glGetProgramInfoLog(program,sizeof log,NULL,log);fprintf(stderr,"%s\n",log);return 1;}
    glGenVertexArrays(1,&vao);aspect_location=glGetUniformLocation(program,"aspect");
    NuiUi *ui=nui_create("<canvas id=scene></canvas><button id=close class=primary>Close</button>",css,800,520);free(css);check(ui!=NULL);
    for(int frame=0;!smoke||frame<8;frame++){
        int event=nui_window_poll(window,ui);check(event>=0);if(event==1||nui_clicked(ui,"close")==1)break;
        NuiColor background={19,32,54,255};check(nui_window_clear(window,background)==1);
        check(nui_window_render(window,ui,draw,NULL)==1);check(nui_window_present(window)==1);check(nui_end_frame(ui)==1);
    }
    glDeleteVertexArrays(1,&vao);glDeleteProgram(program);
    check(nui_destroy(ui)==1);check(nui_window_close(window)==1);return 0;
}
