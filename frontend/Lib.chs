module Lib where

import Data.Int
import Data.Word
import Foreign.Ptr
import Foreign.C.Types

#include "yoke.h"

type Key = Word64

type Symbol = Word32

type Arity = Word16

{# typedef uint64_t Key #}

{# typedef uint32_t Symbol #}

{# typedef uint16_t Arity #}

{# pointer *Project as ProjectPtr foreign newtype #}

{# pointer *Unit as UnitPtr foreign newtype #}

{# fun new_project as ^ {} -> `ProjectPtr' #}

{# fun free_project as ^ { `ProjectPtr' } -> `()' #}

{# fun new_unit as ^ { `ProjectPtr' } -> `UnitPtr' #}

{# fun free_unit as ^ { `UnitPtr' } -> `()' #}

{# fun print_unit as ^ { `UnitPtr' } -> `()' #}

{# fun add_main as ^ { `UnitPtr' } -> `()' #}

{# fun add_data as ^ { `UnitPtr', `Key', `Symbol', `Arity' } -> `()' #}

{# fun jit as ^ { `UnitPtr' } -> `Int32' #}
