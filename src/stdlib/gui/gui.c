#define NK_IMPLEMENTATION
#define NK_INCLUDE_FIXED_TYPES
#define NK_INCLUDE_STANDARD_IO
#define NK_INCLUDE_STANDARD_VARARGS
#define NK_INCLUDE_DEFAULT_ALLOCATOR
#define NK_INCLUDE_VERTEX_BUFFER_OUTPUT
#define NK_INCLUDE_FONT_BAKING
#define NK_INCLUDE_DEFAULT_FONT
#include "nuklear.h"
#include "gui.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
    #define WIN32_LEAN_AND_MEAN
    #include <windows.h>
    #define NK_GDI_IMPLEMENTATION
    #include "nuklear_gdi.h"

    static GdiFont* gdi_font;
    static HWND hwnd;
    static HDC hdc;
    static struct nk_context* ctx;
    static int running = 1;

    static LRESULT CALLBACK 
    WindowProc(HWND wnd, UINT msg, WPARAM wparam, LPARAM lparam) {
        if (msg == WM_CLOSE || msg == WM_DESTROY) {
            fprintf(stderr, "GUI: Window closing (msg=%u)\n", msg);
            PostQuitMessage(0);
            running = 0;
            return 0;
        }
        if (running && ctx && nk_gdi_handle_event(wnd, msg, wparam, lparam)) {
            return 0;
        }
        return DefWindowProcW(wnd, msg, wparam, lparam);
    }
    
    void peel_gui_win32_init(int width, int height, const char* title) {
        WNDCLASSW wc;
        RECT rect = { 0, 0, width, height };
        DWORD style = WS_OVERLAPPEDWINDOW;
        DWORD exstyle = WS_EX_APPWINDOW;
        const wchar_t* class_name = L"PeelNuklearWindowClass";
        
        memset(&wc, 0, sizeof(wc));
        wc.style = CS_DBLCLKS;
        wc.lpfnWndProc = WindowProc;
        wc.hInstance = GetModuleHandleW(NULL);
        wc.hCursor = LoadCursorW(NULL, (LPCWSTR)IDC_ARROW);
        wc.lpszClassName = class_name;
        
        if (!RegisterClassW(&wc) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
            fprintf(stderr, "GUI: RegisterClassW failed (error %lu)\n", GetLastError());
            running = 0;
            return;
        }
        
        AdjustWindowRectEx(&rect, style, FALSE, exstyle);
        
        int w_title = MultiByteToWideChar(CP_UTF8, 0, title, -1, NULL, 0);
        wchar_t* title_w = (wchar_t*)malloc(w_title * sizeof(wchar_t));
        MultiByteToWideChar(CP_UTF8, 0, title, -1, title_w, w_title);

        hwnd = CreateWindowExW(exstyle, wc.lpszClassName, title_w,
            style | WS_VISIBLE, CW_USEDEFAULT, CW_USEDEFAULT,
            rect.right - rect.left, rect.bottom - rect.top,
            NULL, NULL, wc.hInstance, NULL);
            
        free(title_w);
        if (!hwnd) {
            fprintf(stderr, "GUI: CreateWindowExW failed with error %lu\n", GetLastError());
            running = 0;
            return;
        }
        fprintf(stderr, "GUI: Window created successfully (hwnd=%p)\n", (void*)hwnd);
        
        hdc = GetDC(hwnd);
        if (!hdc) {
            fprintf(stderr, "GUI: GetDC failed\n");
            running = 0;
            return;
        }
        
        gdi_font = nk_gdifont_create("Arial", 14);
        if (!gdi_font) {
            fprintf(stderr, "GUI: nk_gdifont_create failed\n");
            running = 0;
            return;
        }
        
        ctx = nk_gdi_init(gdi_font, hdc, width, height);
        if (!ctx) {
            fprintf(stderr, "GUI: nk_gdi_init failed\n");
            running = 0;
            return;
        }
        
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        running = 1;
        fprintf(stderr, "GUI: Init completed successfully\n");
    }
    
    void peel_gui_win32_shutdown() {
        if (hwnd) {
            if (hdc) ReleaseDC(hwnd, hdc);
            DestroyWindow(hwnd);
        }
        nk_gdi_shutdown();
        if (gdi_font) nk_gdifont_del(gdi_font);
        UnregisterClassW(L"NuklearWindowClass", GetModuleHandleW(NULL));
        running = 0;
        ctx = NULL;
    }
    
    int peel_gui_win32_should_close() {
        return !running;
    }
    
    void peel_gui_win32_poll_events() {
        MSG msg;
        nk_input_begin(ctx);
        while (PeekMessageW(&msg, NULL, 0, 0, PM_REMOVE)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        nk_input_end(ctx);
    }
    
    void peel_gui_win32_render(struct nk_context* nk_ctx) {
        (void)nk_ctx;
        if (!running) return;
        nk_gdi_render(nk_rgb(30,30,30));
    }
#else
    /* Placeholder for X11 */
    #include <X11/Xlib.h>
    #include <X11/Xutil.h>
    /* nk_xlib needs some more setup but this is fine for now as a cross-platform proof */
    #define NK_XLIB_IMPLEMENTATION
    #include "nuklear_xlib.h"

    static Display* dpy;
    static Window win;
    static struct nk_context* ctx;
    static int running = 1;

    void peel_gui_x11_init(int width, int height, const char* title) {
        dpy = XOpenDisplay(NULL);
        if (!dpy) return;
        win = XCreateSimpleWindow(dpy, DefaultRootWindow(dpy), 0, 0, width, height, 0, 0, 0);
        XStoreName(dpy, win, title);
        XSelectInput(dpy, win, ExposureMask | KeyPressMask | KeyReleaseMask | ButtonPressMask | ButtonReleaseMask | PointerMotionMask);
        XMapWindow(dpy, win);
        ctx = nk_xlib_init(dpy, win, width, height);
    }

    void peel_gui_x11_shutdown() {
        if (!running) return;
        nk_xlib_shutdown();
        XCloseDisplay(dpy);
        running = 0;
    }

    int peel_gui_x11_should_close() { return !running; }
    void peel_gui_x11_poll_events() {
        XEvent evt;
        nk_input_begin(ctx);
        while (XPending(dpy)) {
            XNextEvent(dpy, &evt);
            nk_xlib_handle_event(&evt);
        }
        nk_input_end(ctx);
    }
    void peel_gui_x11_render(struct nk_context* nk_ctx) {
        (void)nk_ctx;
        if (!running) return;
        nk_xlib_render(win, NK_ANTI_ALIASING_ON, nk_rgb(30,30,30));
    }
#endif

void peel_gui_init(int width, int height, const char* title) {
#ifdef _WIN32
    peel_gui_win32_init(width, height, title);
#else
    peel_gui_x11_init(width, height, title);
#endif
}

void peel_gui_shutdown() {
#ifdef _WIN32
    peel_gui_win32_shutdown();
#else
    peel_gui_x11_shutdown();
#endif
}

int peel_gui_should_close() {
#ifdef _WIN32
    return peel_gui_win32_should_close();
#else
    return peel_gui_x11_should_close();
#endif
}

void peel_gui_poll_events() {
#ifdef _WIN32
    peel_gui_win32_poll_events();
#else
    peel_gui_x11_poll_events();
#endif
}

void peel_gui_render() {
#ifdef _WIN32
    peel_gui_win32_render(ctx);
#else
    peel_gui_x11_render(ctx);
#endif
}

/* Nuklear Wrapper Functions */
int peel_gui_window_begin(const char* title, int x, int y, int w, int h, int flags) {
    if (!ctx) return 0;
    /* Combine default flags: BORDER | MOVABLE | SCALABLE | TITLE */
    if (flags == 0) flags = 1 | 2 | 4 | 512; 
    return nk_begin(ctx, title, nk_rect(x, y, w, h), flags);
}

void peel_gui_window_end() {
    if (ctx) nk_end(ctx);
}

void peel_gui_layout_row_dynamic(float height, int cols) {
    if (ctx) nk_layout_row_dynamic(ctx, height, cols);
}

void peel_gui_label(const char* text, int align) {
    if (ctx) nk_label(ctx, text, align);
}

int peel_gui_button(const char* label) {
    if (!ctx) return 0;
    return nk_button_label(ctx, label);
}
