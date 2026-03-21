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

import SDL_types;

extern(C):

/* This is the OS scheduler timeslice, in milliseconds */
const uint SDL_TIMESLICE	= 10;

/* This is the maximum resolution of the SDL timer on all platforms */
const uint TIMER_RESOLUTION	= 10;	/* Experimentally determined */

/* Get the number of milliseconds since the SDL library initialization.
 * Note that this value wraps if the program runs for more than ~49 days.
 */ 
Uint32 SDL_GetTicks();

/* Wait a specified number of milliseconds before returning */
void SDL_Delay(Uint32 ms);

/* Function prototype for the timer callback function */
alias SDL_TimerCallback = extern(C) Uint32 function(Uint32 interval);

/* New timer API, supports multiple timers
 * Written by Stephane Peter <megastep@lokigames.com>
 */

/* Function prototype for the new timer callback function.
 * The callback function is passed the current timer interval and returns
 * the next timer interval.  If the returned value is the same as the one
 * passed in, the periodic alarm continues, otherwise a new alarm is
 * scheduled.  If the callback returns 0, the periodic alarm is cancelled.
 */
alias SDL_NewTimerCallback = extern(C) Uint32 function(Uint32 interval, void *param);

/* Definition of the timer ID type */
alias void *SDL_TimerID;