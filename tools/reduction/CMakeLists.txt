project(LeanExchangeSmoke)

add_executable(${PROJECT_NAME}
  LeanExchangeSmoke.cxx
)

target_link_libraries(${PROJECT_NAME} PRIVATE
  TKSTEPCore
  TKDE
  TKXSBase
  TKFeat
  TKOffset
  TKFillet
  TKHelix
  TKMesh
  TKBool
  TKBO
  TKPrim
  TKShHealing
  TKTopAlgo
  TKGeomAlgo
  TKBRep
  TKGeomBase
  TKG3d
  TKG2d
  TKMath
  TKernel
)

set_target_properties(${PROJECT_NAME} PROPERTIES
  FOLDER "Tools/Reduction"
)

add_test(NAME ${PROJECT_NAME}
         COMMAND ${PROJECT_NAME})
