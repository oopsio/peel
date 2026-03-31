#ifndef PEEL_GUI_H
#define PEEL_GUI_H

#include <stdint.h>

typedef struct {
    int x;
    int y;
    int width;
    int height;
} peel_gui_rect;

void peel_gui_init(int width, int height, const char* title);
void peel_gui_shutdown();
int peel_gui_should_close();
void peel_gui_poll_events();
void peel_gui_render();

int peel_gui_window_begin(const char* title, int x, int y, int w, int h, int flags);
void peel_gui_window_end();

void peel_gui_layout_row_dynamic(float height, int cols);
void peel_gui_label(const char* text, int align);
int peel_gui_button(const char* label);
void peel_gui_edit_string(char* buffer, int max_len, int* len);

#endif
