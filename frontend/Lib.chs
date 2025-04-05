module Lib where

import Foreign.Ptr
import Foreign.C.Types

#include "yoke.h"

{#pointer *Project as ProjectPtr foreign newtype #}

{#pointer *Unit as UnitPtr foreign newtype #}

{#fun new_project as ^ {} -> `ProjectPtr' #}

{#fun new_unit as ^ { `ProjectPtr' } -> `UnitPtr' #}

{#fun print_unit as ^ { `UnitPtr' } -> `()' #}

{#fun add_main as ^ { `UnitPtr' } -> `()' #}

{#fun add_data as ^ { `UnitPtr', `CULong', `CUInt', `CUShort' } -> `()' #}
