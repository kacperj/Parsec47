/*
    SDL - Simple DirectMedia Layer
    Copyright (C) 1997, 1998, 1999, 2000, 2001  Sam Lantinga

    This library is free software; you can redistribute it and/or
    modify it under the terms of the GNU Library General Public
    License as published by the Free Software Foundation; either
    version 2 of the License, or (at your option) any later version.

    This library is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
    Library General Public License for more details.

    You should have received a copy of the GNU Library General Public
    License along with this library; if not, write to the Free
    Foundation, Inc., 59 Temple Place, Suite 330, Boston, MA  02111-1307  USA

    Sam Lantinga
    slouken@devolution.com
*/

/* Include file for SDL event handling */

import SDL_types;

extern(C):

/* Event enumerations */
enum { SDL_NOEVENT = 0,			/* Unused (do not remove) */
       SDL_QUIT = 12,			/* User-requested quit */
       SDL_SYSWMEVENT = 13,		/* System specific event */
       SDL_EVENT_RESERVEDA = 14,	/* Reserved for future use.. */
       SDL_EVENT_RESERVEDB = 15,	/* Reserved for future use.. */
       SDL_VIDEORESIZE = 16,		/* User resized video mode */
       SDL_VIDEOEXPOSE = 17,		/* Screen needs to be redrawn */
       /* Events SDL_USEREVENT through SDL_MAXEVENTS-1 are for your use */
       SDL_USEREVENT = 24
}

/* The "window resized" event
   When you get this event, you are responsible for setting a new video
   mode with the new width and height.
 */
struct SDL_ResizeEvent {
	Uint8 type;	/* SDL_VIDEORESIZE */
	int w;		/* New width */
	int h;		/* New height */
}

/* The "screen redraw" event */
struct SDL_ExposeEvent {
	Uint8 type;	/* SDL_VIDEOEXPOSE */
}

/* The "quit requested" event */
struct SDL_QuitEvent {
	Uint8 type;	/* SDL_QUIT */
}

/* A user-defined event type */
struct SDL_UserEvent {
	Uint8 type;	/* SDL_USEREVENT through SDL_NUMEVENTS-1 */
	int code;	/* User defined event code */
	void *data1;	/* User defined data pointer */
	void *data2;	/* User defined data pointer */
}

/* General event structure */
union SDL_Event {
	Uint8 type;
	SDL_ResizeEvent resize;
	SDL_ExposeEvent expose;
	SDL_QuitEvent quit;
	SDL_UserEvent user;
}

/* Polls for currently pending events, and returns 1 if there are any pending
   events, or 0 if there are none available.  If 'event' is not NULL, the next
   event is removed from the queue and stored in that area.
 */
int SDL_PollEvent(SDL_Event *event);

const uint SDL_DISABLE	= 0;
const uint SDL_ENABLE	= 1;
